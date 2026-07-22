use std::{borrow::Cow, marker::PhantomData};

use oauth2::http::Uri;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::server::{SessionId, interface::redirect::OodInternalRedirect};

pub mod bridge;
pub mod elements;
pub mod page;
pub mod redirect;

pub enum OodReplyType {
    Payload(serde_json::Value), // don't want to deal with cache right now...
    Error(String),              // outside doesn't need to know error type exactly
    Finished,
    InternalRedirect(InternalRedirectType),
    ExternalRedirect(Cow<'static, str>),
}

pub enum InternalRedirectType {
    NewPage(Box<dyn OodInternalRedirect>),
    Session(SessionId),
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

pub trait OodActionHasSummary: OodAction {
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
pub trait OodActionHasNoSummary: OodAction {
    fn new<'a>(item: &'a Self::Item) -> OodReply<'a, Self>
    where
        Self: Sized,
    {
        OodReply {
            action: Self::NAME,
            summary: "",
            item,
        }
    }
}

impl<T: OodAction<ActionType = HasSummary>> OodActionHasSummary for T {}
impl<T: OodAction<ActionType = NoSummary>> OodActionHasNoSummary for T {}

pub struct HasSummary;
pub struct NoSummary;
pub trait OodActionType {}
impl OodActionType for HasSummary {}
impl OodActionType for NoSummary {}

pub trait OodAction {
    const NAME: &'static str;
    type Item: ?Sized + Serialize; // needed to set type Item = str
    type Reply: DeserializeOwned;
    type ActionType: OodActionType;
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
