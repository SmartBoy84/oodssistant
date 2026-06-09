use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
#[error("google api error: {message}")]
pub struct GCalApiError {
    pub code: u16,
    pub message: String,
    pub status: Option<String>,
    #[serde(default)]
    pub errors: Vec<GCalApiErrorItem>,
    // pub details: Option<Vec<Value>>, // too complicated + this is moreso for dynamically-typed langs
}

#[derive(Debug, Deserialize)]
pub struct GCalApiErrorItem {
    pub message: String,
    pub domain: String, // i.e., where you need this
    pub reason: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default, rename = "locationType")]
    pub location_type: Option<String>, // e.g., header - must add to header
}
