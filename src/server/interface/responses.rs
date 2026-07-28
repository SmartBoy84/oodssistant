use std::{marker::PhantomData, str::Utf8Error};

use serde::{
    Deserialize,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::server::interface::{OodParse, OodParseWithContentType, OodPayloadParseError};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SerdeOptionalResponse<T> {
    Empty(SerdeEmptyResponse), // AYYYY, put this *first* to have "" -> EmptyString
    Res(T),
}

#[derive(Debug)]
pub enum SerdeEmptyResponse {
    Null,
    None,
    EmptyString,
}

#[derive(Debug)]
pub struct EmptyResponse;

#[derive(Debug, Error)]
#[error("not empty")]
pub struct NotEmpty;

#[derive(Debug)]
pub struct JsonPayload<'de, T: Deserialize<'de>> {
    inner: bytes::Bytes,
    _lifetime: PhantomData<&'de T>,
}

impl OodParse for () {
    type E = NotEmpty; // in case user wants to enforce that return is empty (if not, just discard!)
    type O<'a> = ();
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        _: &'a Option<mime::Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        (body.len() == 0).then_some(()).ok_or(NotEmpty.into())
    }
}

impl OodParseWithContentType for str {
    type E = Utf8Error;
    type O<'a>
        = &'a str
    where
        Self: 'a;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a mime::Mime,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        // for now enforce ONLY text - even though things like application/{json, xml etc} may also be valid utf-8/16
        if content_type.type_() != mime::TEXT {
            return Err(OodPayloadParseError::InvalidContentType(
                content_type.clone(),
            ));
        }
        str::from_utf8(body).map_err(Into::into)
    }
}

// impl<'de, T: Deserialize<'de>> OodParseWithContentType for JsonPayload<'de, T> {
//     type E = serde_json::Error;
//     type O<'a> = T;

//     fn ood_try_from<'a>(
//         body: &'a bytes::Bytes,
//         content_type: &'a mime::Mime,
//     ) -> Result<Self::O<'de>, OodPayloadParseError<Self::E>>
//     where
//         'a: 'de,
//     {
//         // strictly application JSON *only*
//         if content_type != &mime::APPLICATION_JSON {
//             return Err(OodPayloadParseError::InvalidContentType(
//                 content_type.clone(),
//             ));
//         }
//         serde_json::from_slice(body).map_err(Into::into)
//     }
// }

// could match any string - but want to enforce that incoming data should be empty to not confuse users (me, myself and I!)
impl<'de> Deserialize<'de> for SerdeEmptyResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EmptyResponseVisitor;
        impl<'de> Visitor<'de> for EmptyResponseVisitor {
            type Value = SerdeEmptyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("null or empty string (`\"\"`)")
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SerdeEmptyResponse::Null)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "" => Ok(SerdeEmptyResponse::EmptyString),
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
                Ok(SerdeEmptyResponse::None)
            }
        }

        deserializer.deserialize_any(EmptyResponseVisitor)
    }
}
