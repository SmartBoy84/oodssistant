use std::borrow::Cow;

use chrono::{DateTime, Utc};
use restman_rs::{POST, endpoint, request::RequestConfig, request_part};
use serde::{Deserialize, Serialize};

use crate::bark::payload::BarkPayload;

pub struct BarkServer(pub String);
impl restman_rs::Server for BarkServer {}

impl restman_rs::ConstServer for BarkServer {
    const ROOT: &str = "https://api.day.app";
}

impl restman_rs::DynamicServer for BarkServer {
    fn get_root(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct BarkRes {
    pub code: usize,
    pub message: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

// naive unwrapping
impl Into<Result<BarkRes, BarkRes>> for BarkRes {
    fn into(self) -> Result<BarkRes, BarkRes> {
        if self.code == 200 {
            Ok(self)
        } else {
            Err(self)
        }
    }
}

#[derive(Serialize)]
pub struct BarkConfig<'a> {
    pub key: Cow<'a, str>,
}
impl<'a> RequestConfig for BarkConfig<'a> {}
trait HasKey {
    fn key(&self) -> &str;
}
impl HasKey for BarkConfig<'_> {
    fn key(&self) -> &str {
        &self.key
    }
}

request_part!(BarkRoot, "", (), HasKey, key);
endpoint!(BarkServer, pub BarkRequest, "", BarkRoot, BarkRes, (), BarkPayload, POST);
