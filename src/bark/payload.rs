use bon::Builder;
use restman_rs::request::QueryPayload;
use serde::Serialize;
use serde_with::skip_serializing_none;

// Push notification payload for Bark
#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Builder)]
// so that I don't need to manually convert to string
// bit scummy because I make it seem to the user that this borrows when it really doesn't...
#[builder(on(String, into))]
pub struct BarkPayload {
    pub title: Option<String>, // required
    pub subtitle: Option<String>,
    pub body: Option<String>,
    pub markdown: Option<String>,

    pub level: Option<PushLevel>,

    // as per docs
    #[builder(default = 5)]
    pub volume: u8,

    pub badge: Option<u32>,

    #[serde(with = "bool01_opt")]
    pub call: Option<bool>,

    #[serde(with = "bool01_opt")]
    pub auto_copy: Option<bool>,

    pub copy: Option<String>,
    pub sound: Option<String>,
    pub icon: Option<String>,
    pub image: Option<String>,

    pub group: Option<String>,
    pub ciphertext: Option<String>,

    #[serde(with = "bool01_opt")]
    pub is_archive: Option<bool>,

    pub ttl: Option<u64>,
    pub url: Option<String>,
    pub action: Option<PushAction>,
    pub id: Option<String>,

    #[serde(with = "bool01_opt")]
    pub delete: Option<bool>,
}

impl QueryPayload for BarkPayload {}

/// Notification interruption level
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub enum PushLevel {
    /// Critical alert (overrides silent mode)
    Critical,
    /// Default alert (lights screen immediately)
    Active,
    /// Time-sensitive notification
    TimeSensitive,
    /// Passive notification (no alert)
    Passive,
}

/// Notification action behavior
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PushAction {
    /// Show alert dialog when opened
    Alert,
}

pub mod bool01_opt {
    use serde::Serializer;

    pub fn serialize<S>(v: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(true) => s.serialize_u8(1),
            Some(false) => s.serialize_u8(0),
            None => s.serialize_none(),
        }
    }
}
