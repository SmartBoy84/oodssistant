pub mod responses;

use std::marker::PhantomData;

use mime::Mime;
use thiserror::Error;

use crate::server::interface::{IntOodAppErr, internal::OodAction};

#[derive(Debug, Error)]
pub enum OodPayloadParseError<E: std::error::Error> {
    #[error("missing content-type header")]
    NoContentTypeHeader,
    #[error("Do not support {0}")]
    InvalidContentType(Mime),
    #[error(transparent)]
    ParseError(#[from] E),
}

#[derive(Debug)]
pub struct OodResponse {
    pub body: bytes::Bytes,
    pub content_type: Option<Mime>,
}

pub struct OodPayloadParser<A: OodAction> {
    inner: OodResponse,
    _target: PhantomData<A>,
}

impl<A: OodAction> OodPayloadParser<A> {
    pub fn new(inner: OodResponse) -> Self {
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
        let OodResponse { body, content_type } = &self.inner;
        A::Reply::ood_try_from(body, content_type).map_err(IntOodAppErr::ExternalParseErr::<A>) // this is an EXTERNAL error
    }
}

pub trait OodParse {
    type E: std::error::Error;
    type O<'a>;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a Option<Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>>;
}

pub trait OodParseWithContentType {
    type E: std::error::Error;
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