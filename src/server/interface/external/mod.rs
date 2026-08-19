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
    pub body: bytes::Bytes, // can convert this to_vec and it will NOT copy because it will (should) be exclusively owned here
    pub content_type: Option<Mime>,
}

pub struct OodPayloadParser<A: OodAction> {
    inner: OodResponse,
    _target: PhantomData<A>,
}

impl<'a, A: OodAction + 'a> OodPayloadParser<A> {
    pub fn new(inner: OodResponse) -> Self {
        Self {
            inner,
            _target: PhantomData,
        }
    }

    pub fn p(self) -> Result<<A::Reply as OodParse>::O, IntOodAppErr<A>> {
        let OodResponse { body, content_type } = self.inner;
        // NOTE; at this point, Bytes ref count has to be 1 - exclusive ownership - meaning bytes::Bytes can be decomposed into the internal buffer without copying
        A::Reply::ood_try_from(body, content_type).map_err(IntOodAppErr::ExternalParseErr::<A>) // this is an EXTERNAL error
    }
}

pub trait OodParse {
    type E: std::error::Error;
    type O;
    fn ood_try_from(
        body: bytes::Bytes, // indicate requirement of exclusive ownership
        content_type: Option<Mime>,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>>;
}

pub trait OodParseWithContentType {
    type E: std::error::Error;
    type O;
    fn ood_try_from(
        body: bytes::Bytes,
        content_type: Mime,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>>;
}

impl<T: OodParseWithContentType> OodParse for T {
    type E = <Self as OodParseWithContentType>::E;
    type O = <Self as OodParseWithContentType>::O;
    fn ood_try_from(
        body: bytes::Bytes,
        content_type: Option<Mime>,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>> {
        let Some(content_type) = content_type else {
            return Err(OodPayloadParseError::NoContentTypeHeader);
        };
        <Self as OodParseWithContentType>::ood_try_from(body, content_type)
    }
}
