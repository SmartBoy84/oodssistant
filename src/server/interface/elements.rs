use std::{marker::PhantomData, time::Duration};

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::server::interface::OodAction;

#[derive(Debug)]
pub enum EmptyResponse {
    Null,
    None,
    EmptyString,
}

// this is **NOT** the same as b.redirect() -> that is an *internal* redirect, this action instructs the device to open this URI in whatever external application
pub struct OodOpenUri;
impl OodAction for OodOpenUri {
    const NAME: &'static str = "uri";
    type Item = str;
    type Reply = EmptyResponse; // iOS shortcuts won't forget this, but don't leave me hanging on other things!!
}

pub struct OodInfo;
impl OodAction for OodInfo {
    const NAME: &'static str = "info";
    type Item = str; // interesting! we do this here, because we always use &Item (with &str it would become &&str)
    type Reply = EmptyResponse;
}

pub struct OodButtonList<T>(PhantomData<T>);

impl<T> OodAction for OodButtonList<T>
where
    T: Serialize + AsRef<str>,
{
    const NAME: &'static str = "button";
    type Item = [T]; // (name, return value)
    type Reply = String; // shortcut limitation/simplification
}
pub struct OodTimer; // start a timer on the device
#[derive(Serialize)]
pub struct Seconds(u64);
impl From<Duration> for Seconds {
    fn from(value: Duration) -> Self {
        Self(value.as_secs())
    }
}
impl OodAction for OodTimer {
    const NAME: &'static str = "timer";
    type Item = Option<Seconds>; // None - deactivate timer
    type Reply = EmptyResponse;
}

pub struct OodTextInput<'a>(PhantomData<&'a str>);

impl<'a> OodAction for OodTextInput<'a> {
    const NAME: &'static str = "text_input";
    type Item = str; // default value (if editing)
    type Reply = String; // shortcut limitation/simplification
}

// could match any string - but want to enforce that incoming data should be empty to not confuse users (me, myself and I!)
impl<'de> Deserialize<'de> for EmptyResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EmptyResponseVisitor;
        impl<'de> Visitor<'de> for EmptyResponseVisitor {
            type Value = EmptyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("null or empty string (`\"\"`)")
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(EmptyResponse::Null)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "" => Ok(EmptyResponse::EmptyString),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Str(v),
                        &"an empty string or null",
                    )),
                }
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(EmptyResponse::None)
            }
        }

        deserializer.deserialize_any(EmptyResponseVisitor)
    }
}
