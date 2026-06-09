use std::collections::HashMap;

use restman_rs::{GET, endpoint};
use serde::Deserialize;

use crate::gcal::{
    GCalServer,
    api::{GCalApiRes, request::V3},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorEntry {
    pub background: String,
    pub foreground: String,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorsRes {
    #[serde(rename = "event")]
    pub colors: HashMap<String, ColorEntry>,
}

endpoint!(GCalServer, pub ColorsGet, "colors", V3, GCalApiRes<ColorsRes>, (), (), GET);