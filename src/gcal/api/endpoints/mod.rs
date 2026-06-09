use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod calendar_list;
pub mod events;
pub mod colors;

// TODO; I should implement sync token, but I will just have to manually re-sync for now

// for /list endpoints
#[derive(Deserialize)]
pub struct ListRes<T> {
    // kind: String, // don't need it
    pub etag: String, // if two requests have same etag -> they are the same! i.e., the state did not change in between (etag is hash of last modified)
    pub next_page_token: Option<String>, // if there are multiple pages
    pub items: Vec<T>,
    // for events list there are several other descriptors, but they don't matter for this application (at least not in this state)
}

#[skip_serializing_none] // remember to add before deriving serialize
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalDateTime {
    date_time: chrono::DateTime<Utc>,
    timezone: Option<String>, // maybe needed
    date: Option<chrono::NaiveDate>, // for full-day events
}

impl From<DateTime<Utc>> for CalDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        CalDateTime {
            date_time: value,
            timezone: None,
            date: None,
        }
    }
}

// will not implement GET - for simplicity I will get the entire list everytime
