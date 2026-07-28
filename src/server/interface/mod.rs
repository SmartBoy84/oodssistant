use std::{borrow::Cow, error::Error, fmt::Debug, marker::PhantomData};

use mime::Mime;
use serde::Serialize;
use thiserror::Error;

use crate::server::{SessionId, interface::bridge::OodBridge};

pub mod bridge;
pub mod elements;
pub mod page;
pub mod responses;
// pub mod redirect;

#[derive(Debug, Error)]
pub enum OodPayloadParseError<E: Error> {
    #[error("missing content-type header")]
    NoContentTypeHeader,
    #[error("invalid content type")]
    InvalidContentType(Mime),
    #[error(transparent)]
    ParseError(#[from] E),
}

#[derive(Debug)]
pub struct OodPayload {
    pub body: bytes::Bytes,
    pub content_type: Option<Mime>,
}

// this takes OodBridge so we can report errors in here as well
pub struct OodPayloadParser<'a, A: OodAction> {
    bridge: &'a mut OodBridge,
    inner: OodPayload,
    _target: PhantomData<A>,
}

impl<'a, A: OodAction> OodPayloadParser<'a, A> {
    pub fn new(inner: OodPayload, bridge: &'a mut OodBridge) -> Self {
        Self {
            bridge,
            inner,
            _target: PhantomData,
        }
    }

    // single-letter ("parse") for convenience
    /*
    For future me this pattern exists so I can do (where b is Bridge), b.cf(...).p
    cf returns a Parser containing Bytes and I can do .p() immediately after because those bytes are "temporarily stored" in the previous activation stack which persists until p() finishes
    this is really convenient because in 95% of cases I send a response to the client and immediately banch on the reply - i.e., I rarely need to "remember" the response
     */

    // do not use 'a because I don't want to tie output lifetime to bridge
    pub async fn p<'b>(&'b mut self) -> Result<<A::Reply as OodParse>::O<'b>, IntOodAppErr<A>> {
        let OodPayload { body, content_type } = &self.inner;
        let r =
            A::Reply::ood_try_from(body, content_type).map_err(IntOodAppErr::ExternalParseErr::<A>); // this is an EXTERNAL error
        self.bridge.err_wrapper(r).await
    }
}

pub trait OodParse {
    type E: Error;
    type O<'a>;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a Option<Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>>;
}

pub trait OodParseWithContentType {
    type E: Error;
    type O<'a>;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a Mime,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>>;
}

impl<T: OodParseWithContentType> OodParse for T {
    type E = <Self as OodParseWithContentType>::E;
    type O<'a> = <Self as OodParseWithContentType>::O<'a>;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a Option<Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        let Some(content_type) = content_type else {
            return Err(OodPayloadParseError::NoContentTypeHeader);
        };
        <Self as OodParseWithContentType>::ood_try_from(body, content_type)
    }
}

pub enum OodReplyType {
    Payload(bytes::Bytes), // don't want to deal with cache right now...
    Err(ExtOodAppErr),
    Finished,
    InternalRedirect(SessionId),
    ExternalRedirect(Cow<'static, str>),
}

// this is the internal error type (inside the handler)
#[derive(Debug, Error)]
pub enum IntOodAppErr<A: OodAction> {
    #[error(transparent)]
    InternalParseErr(serde_json::Error), // internal is always json for now
    #[error(transparent)]
    ExternalParseErr(OodPayloadParseError<<A::Reply as OodParse>::E>),
}

// this is the external error type (for debugging purposes only) - so I don't have to worry about actually logging (and can just use ?)
#[derive(Debug, Error, Clone)]
pub enum ExtOodAppErr {
    #[error("external parse error")]
    ExternalParseError(Box<str>),

    #[error("internal parse error")]
    InternalParseError(Box<str>), // internal is always json for now
    #[error("channel closed")]
    ChannelClosed,
}

impl<A: OodAction> From<&IntOodAppErr<A>> for ExtOodAppErr {
    fn from(value: &IntOodAppErr<A>) -> Self {
        match value {
            IntOodAppErr::InternalParseErr(e) => {
                ExtOodAppErr::InternalParseError(e.to_string().into_boxed_str())
            }
            IntOodAppErr::ExternalParseErr(e) => {
                ExtOodAppErr::ExternalParseError(e.to_string().into_boxed_str())
            }
        }
    }
}

pub trait OodActionHasSummary: OodAction {
    fn new<'a>(
        summary: &'a <Self::ActionType as OodActionType>::Summary,
        item: &'a Self::Item,
    ) -> OodReply<'a, Self>
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
        Self::ActionType: OodActionType<Summary = ()>,
    {
        OodReply {
            action: Self::NAME,
            summary: &(),
            item,
        }
    }
}

impl<T: OodAction<ActionType = NoSummary>> OodActionHasNoSummary for T {}

pub struct HasSummary<T: ?Sized>(PhantomData<T>);
pub struct NoSummary;
pub trait OodActionType {
    type Summary: ?Sized + Serialize + Debug;
}
impl<S: ?Sized + Serialize + Debug> OodActionType for HasSummary<S> {
    type Summary = S;
}
impl OodActionType for NoSummary {
    type Summary = ();
}

impl<S: ?Sized + Serialize, T: OodAction<ActionType = HasSummary<S>>> OodActionHasSummary for T {}

pub trait OodAction {
    const NAME: &'static str;
    type Item: ?Sized + Serialize; // needed to set type Item = str
    type Reply: OodParse;
    type ActionType: OodActionType;
}

#[derive(Serialize, Debug)]
pub struct OodReply<'a, T: OodAction> {
    action: &'static str,
    summary: &'a <T::ActionType as OodActionType>::Summary,
    item: &'a T::Item,
}
