use std::fmt::Display;

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::server::interface::elements::{EmptyResponse, OodFilePath};

/// OodFilePath serialisation
const SHORTCUT_FILE_EXT: &str = ".txt";

// tiiiiny bit of unsafe but it's alright!
impl<T: AsRef<str> + ?Sized> From<&T> for &OodFilePath {
    fn from(value: &T) -> Self {
        // SAFETY: `#[repr(transparent)]`
        unsafe { &*(value.as_ref() as *const _ as *const OodFilePath) }
    }
}

impl From<&OodFilePath> for &str {
    fn from(value: &OodFilePath) -> Self {
        // SAFETY: `#[repr(transparent)]`
        unsafe { &*(value as *const _ as *const str) }
    }
}

// implement Display then collect_str to minimise allocations
impl Display for OodFilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())?; // push the path first
        if !self.0.ends_with(SHORTCUT_FILE_EXT) {
            f.write_str(SHORTCUT_FILE_EXT)?; // push the required extension if needed
            // This is because iOS shortcuts implicitly adds .txt so enforce it here
        }
        Ok(())
    }
}

impl Serialize for OodFilePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self) // allows efficient allocation
        // e.g., serde_json will use the Display implementation to directly push the string to its buffer
        // this way when using a const str with the incorrect extension, no additional allocations occur
        // the display implementation pushes the const str then the correct file extension
    }
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
