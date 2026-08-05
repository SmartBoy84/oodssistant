use std::{borrow::Cow, fmt::Debug};

use serde::Serialize;
use thiserror::Error;

use crate::server::{
    OodPayload, SessionId,
    interface::{
        external::{OodParse, OodPayloadParseError},
        internal::{OodAction, OodActionType, ToOodItemHeader},
    },
    request::OodPayloadStreamer,
};

pub mod bridge;
pub mod elements;
pub mod external;
pub mod internal;
pub mod page;
// pub mod redirect;

pub enum OodReplyType {
    Payload(OodPayload), // don't want to deal with cache right now...
    Finished,
    InternalRedirect(SessionId),
    ExternalRedirect(Cow<'static, str>),
}

#[derive(Debug, Error)]
pub enum IntOodParseErr<A: OodAction> {
    #[error(transparent)]
    ItemParseErr(<A::Item as ToOodItemHeader<A::Item>>::Err),
    #[error("invalid header value")]
    InvalidHeaderValue,
    #[error(transparent)]
    PayloadErr(Box<<<A::ActionType as OodActionType>::Data as OodPayloadStreamer>::StreamErr>),
}

// this is the internal error type (inside the handler)
#[derive(Error)]
pub enum IntOodAppErr<A: OodAction> {
    #[error(transparent)]
    InternalParseErr(IntOodParseErr<A>), // can't figure out how to get ...::E into here as well
    #[error(transparent)]
    ExternalParseErr(OodPayloadParseError<<A::Reply as OodParse>::E>),
}

impl<A: OodAction> From<IntOodParseErr<A>> for IntOodAppErr<A> {
    fn from(value: IntOodParseErr<A>) -> Self {
        Self::InternalParseErr(value)
    }
}

// this is the external error type (for debugging purposes only) - so I don't have to worry about actually logging (and can just use ?)
#[derive(Debug, Error, Clone)]
pub enum ExtOodAppErr {
    #[error("external parse error")]
    ExternalParseError(String), // error reading the reply (e,.g., parsing)

    #[error("internal parse error")]
    InternalParseError(String), // error creating the response payload
    #[error("channel closed")]
    ChannelClosed,
}

#[derive(Serialize)]
pub struct OodPayloadItem<'a, T: Serialize> {
    item: &'a T,
}

impl<A: OodAction> From<IntOodAppErr<A>> for ExtOodAppErr {
    fn from(value: IntOodAppErr<A>) -> Self {
        match value {
            IntOodAppErr::InternalParseErr(e) => ExtOodAppErr::InternalParseError(e.to_string()),
            IntOodAppErr::ExternalParseErr(e) => ExtOodAppErr::ExternalParseError(e.to_string()),
        }
    }
}

impl<A: OodAction> From<&IntOodAppErr<A>> for ExtOodAppErr {
    fn from(value: &IntOodAppErr<A>) -> Self {
        match value {
            IntOodAppErr::InternalParseErr(e) => ExtOodAppErr::InternalParseError(e.to_string()),
            IntOodAppErr::ExternalParseErr(e) => ExtOodAppErr::ExternalParseError(e.to_string()),
        }
    }
}
