use std::marker::PhantomData;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::server::{handlers::SessionId, interface::redirect::OodInternalRedirect};

pub mod bridge;
pub mod elements;
pub mod page;
pub mod redirect;

pub enum OodReplyType {
    Payload(serde_json::Value), // don't want to deal with cache right now...
    Error(String),              // outside doesn't need to know error type exactly
    Finished,
    InternalRedirect(Box<dyn OodInternalRedirect>),
    ExternalRedirect(SessionId),
}

#[derive(Debug, Error)]
pub enum OodAppErr {
    #[error("external parse error")]
    ExternalParseError(serde_json::Error),

    #[error("internal parse error")]
    InternalParseError(serde_json::Error),

    #[error("failed to match")]
    FailedMatch,
}

pub trait OodAction {
    const NAME: &'static str;
    type Item: ?Sized + Serialize; // needed to set type Item = str
    type Reply: DeserializeOwned;

    fn new<'a>(summary: &'a str, item: &'a Self::Item) -> OodReply<'a, Self>
    where
        Self: Sized,
    {
        OodReply {
            action: Self::NAME,
            summary,
            item,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct OodRes<T: OodAction> {
    pub res: T::Reply,
    #[serde(skip)]
    _p: PhantomData<T>,
}

#[derive(Serialize, Debug)]
pub struct OodReply<'a, T: OodAction> {
    action: &'static str,
    summary: &'a str,
    item: &'a T::Item,
}
