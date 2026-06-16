use std::borrow::Cow;

use serde::Deserialize;
use serde_with::DeserializeFromStr;
use strum::{Display, EnumString, VariantNames};

use crate::gcal::api::error::GCalApiError;

pub mod endpoints;
pub mod error;
pub mod request;

#[derive(Debug, Display, VariantNames, DeserializeFromStr, EnumString, PartialEq, Eq, Clone)]
pub enum EventColour {
    #[strum(serialize = "2")]
    Green, // "sage" - light green
    #[strum(serialize = "7")]
    Blue, // "peacock" - light blue
    #[strum(serialize = "6")]
    Red, // "tangerine" - light red
    #[strum(serialize = "3")]
    Purple, // "grape" - dark purple
    #[strum(serialize = "5")]
    Yellow, // "banana" - yellow
}

#[bon_macro::bon_config]
pub struct GCalConfig<'a> {
    calendar_id: Cow<'a, str>,
    event_id: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GCalApiRes<T> {
    Ok(T),
    Err { error: GCalApiError },
}

impl<T> GCalApiRes<T> {
    pub fn into_result(self) -> Result<T, GCalApiError> {
        match self {
            GCalApiRes::Ok(value) => Ok(value),
            GCalApiRes::Err { error } => Err(error),
        }
    }
}
