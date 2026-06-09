use std::borrow::Cow;

use serde::Deserialize;

use crate::gcal::api::error::GCalApiError;

pub mod colors;
pub mod endpoints;
pub mod error;
pub mod request;

pub const COLOR_GREEN_ID: &str = "2"; // "sage" - light green
pub const COLOR_BLUE_ID: &str = "7"; // "peacock" - light blue
pub const COLOR_RED_ID: &str = "6"; // "tangerine" - light red
pub const COLOR_PURPLE_ID: &str = "3"; // "grape" - dark purple
pub const COLOR_YELLOW_ID: &str = "5"; // "banana" - yellow

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
