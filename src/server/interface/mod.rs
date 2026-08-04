use std::{borrow::Cow, error::Error, fmt::Debug, marker::PhantomData};

use mime::Mime;
use oauth2::http::HeaderValue;
use serde::Serialize;
use thiserror::Error;

use crate::server::{OodPayloadResponder, SessionId};

pub mod bridge;
pub mod elements;
pub mod page;
pub mod responses;
// pub mod redirect;

#[derive(Debug, Error)]
pub enum OodPayloadParseError<E: Error> {
    #[error("missing content-type header")]
    NoContentTypeHeader,
    #[error("Do not support {0}")]
    InvalidContentType(Mime),
    #[error(transparent)]
    ParseError(#[from] E),
}

#[derive(Debug)]
pub struct OodPayload {
    pub body: bytes::Bytes,
    pub content_type: Option<Mime>,
}

pub struct OodPayloadParser<A: OodAction> {
    inner: OodPayload,
    _target: PhantomData<A>,
}

impl<'a, A: OodAction> OodPayloadParser<A> {
    pub fn new(inner: OodPayload) -> Self {
        Self {
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
    pub fn p<'b>(&'b self) -> Result<<A::Reply as OodParse>::O<'b>, IntOodAppErr<A>> {
        let OodPayload { body, content_type } = &self.inner;
        A::Reply::ood_try_from(body, content_type).map_err(IntOodAppErr::ExternalParseErr::<A>) // this is an EXTERNAL error
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

impl<T: OodParseWithContentType + ?Sized> OodParse for T {
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
    Payload(Box<dyn OodPayloadResponder>), // don't want to deal with cache right now...
    Finished,
    InternalRedirect(SessionId),
    ExternalRedirect(Cow<'static, str>),
}

#[derive(Debug, Error)]
pub enum IntOodParseErr {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid header value")]
    InvalidHeaderValue(String),
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
    ExternalParseError(String), // error reading the reply (e,.g., parsing)

    #[error("internal parse error")]
    InternalParseError(String), // error creating the response payload
    #[error("channel closed")]
    ChannelClosed,
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

fn new_reply<'a, A: OodAction>(
    data: &'a <A::ActionType as OodActionType>::Data,
    item: &'a A::Item,
) -> Result<OodReply<'a, A>, IntOodParseErr> {
    let val = serde_json::to_string(item)?;
    Ok(OodReply {
        action: HeaderValue::from_static(A::NAME),
        data,
        item: HeaderValue::from_str(&val).map_err(|_| IntOodParseErr::InvalidHeaderValue(val))?,
    })
}

pub trait OodActionHasData: OodAction {
    fn new<'a>(
        data: &'a <Self::ActionType as OodActionType>::Data,
        item: &'a Self::Item,
    ) -> Result<OodReply<'a, Self>, IntOodParseErr>
    where
        Self: Sized,
    {
        new_reply::<Self>(data, item)
    }
}
pub trait OodActionHasNoData: OodAction {
    fn new<'a>(item: &'a Self::Item) -> Result<OodReply<'a, Self>, IntOodParseErr>
    where
        Self: Sized,
        Self::ActionType: OodActionType<Data = ()>,
    {
        new_reply::<Self>(&(), item)
    }
}

impl<T: OodAction<ActionType = NoData>> OodActionHasNoData for T {}

pub struct HasData<T: ?Sized>(PhantomData<T>);
pub struct NoData;
pub trait OodActionType {
    type Data: ?Sized + Serialize + Debug;
}
impl<S: ?Sized + Serialize + Debug> OodActionType for HasData<S> {
    type Data = S;
}
impl OodActionType for NoData {
    type Data = ();
}

impl<S: ?Sized + Serialize, T: OodAction<ActionType = HasData<S>>> OodActionHasData for T {}

pub trait OodAction {
    const NAME: &'static str;
    type Item: ?Sized + Serialize; // `?Sized` to allow using &str, `Serialize` as a convenience method

    type Reply: OodParse + ?Sized;
    type ActionType: OodActionType;
}

// trait allows for more dynamic types as well (e.g., dynamic action and summary) - this is the most basic implementation
#[derive(Debug)]
pub struct OodReply<'a, T: OodAction> {
    // parsed header values
    action: HeaderValue,
    item: HeaderValue,

    // data
    data: &'a <T::ActionType as OodActionType>::Data,
}
