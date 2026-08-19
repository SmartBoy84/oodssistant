pub mod items;
pub mod payloads;

use std::{marker::PhantomData, sync::Arc};

use oauth2::http::HeaderValue;
use thiserror::Error;

use crate::server::{
    interface::{IntOodAppErr, IntOodParseErr, external::OodParse},
    request::{OodPayload, OodPayloadGetter, OodPayloadStreamer},
};

#[derive(Debug, Error)]
pub enum OodItemErr<T: TryToOodBytes> {
    InvalidHeader,
    ItemParseErr(T::E),
}

pub trait TryToOodBytes {
    type E: std::error::Error;
    type O<'a>
    where
        Self: 'a;

    // please note that the following is after MANY conceptual iterations, *everything* is intentional (e.g., `s: Self` instead of `self`)
    fn to_ood_bytes<'a>(s: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a; // FINALLY figured it out: `s: Self` ()
}

fn new_reply<'a, A: OodAction, T>(
    data: <A::ActionType as OodActionType>::Data,
    item: T,
) -> Result<LinkedOodReply<A>, IntOodAppErr<A>>
where
    A: Sized,
    T: Into<<A::Item as TryToOodBytes>::O<'a>>,
    <A as OodAction>::Item: 'a,
{
    let item = HeaderValue::from_maybe_shared(
        A::Item::to_ood_bytes(item.into()).map_err(IntOodParseErr::ItemParseErr)?,
    )
    .map_err(|_| IntOodParseErr::InvalidHeaderValue)?;

    Ok(LinkedOodReply::new(OodReply {
        action: HeaderValue::from_static(A::NAME),
        data,
        item,
    }))
}

pub trait OodActionHasData: OodAction {
    fn new<'a, T, K>(data: T, item: K) -> Result<LinkedOodReply<Self>, IntOodAppErr<Self>>
    where
        <Self::ActionType as OodActionType>::Data: From<T>,
        Self: Sized,
        K: Into<<Self::Item as TryToOodBytes>::O<'a>>,
        <Self as OodAction>::Item: 'a,
    {
        new_reply::<_, _>(data.into(), item)
    }
}

pub trait OodActionHasNoData: OodAction {
    fn new<'a, K>(item: K) -> Result<LinkedOodReply<Self>, IntOodAppErr<Self>>
    where
        Self: Sized,
        Self::ActionType: OodActionType<Data = ()>,
        <Self::ActionType as OodActionType>::Data: From<()>,
        K: Into<<Self::Item as TryToOodBytes>::O<'a>>,
        <Self as OodAction>::Item: 'a,
    {
        new_reply::<_, _>((), item)
    }
}

impl<T: OodAction<ActionType = NoData>> OodActionHasNoData for T {}

pub struct HasData<T>(PhantomData<T>);
pub struct NoData;
pub trait OodActionType {
    type Data: OodPayloadStreamer;
}
impl<S: OodPayloadStreamer> OodActionType for HasData<S> {
    type Data = S;
}
impl OodActionType for NoData {
    type Data = ();
}

impl<S, T: OodAction<ActionType = HasData<S>>> OodActionHasData for T {}

pub trait OodAction {
    const NAME: &'static str;
    type Item: TryToOodBytes;
    // include a lifetime to allow for borrowing

    type Reply: OodParse;
    type ActionType: OodActionType;
}

// trait allows for more dynamic types as well (e.g., dynamic action and summary) - this is the most basic implementation
pub struct OodReply<S: OodPayloadStreamer> {
    // parsed header values
    action: HeaderValue,
    item: HeaderValue,

    // data
    data: S,
}

pub struct LinkedOodReply<T: OodAction> {
    inner: OodPayload,
    _t: PhantomData<fn(&T)>,
}
impl<T: OodAction> Clone for LinkedOodReply<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(), // Clone is very cheap assuming that cloning OodPayload is cheap (it is - cloning Arc)
            _t: PhantomData,
        }
    }
}

impl<T: OodAction> LinkedOodReply<T> {
    fn new<S: OodPayloadStreamer>(r: OodReply<S>) -> Self {
        Self {
            inner: Arc::new(r),
            _t: PhantomData,
        }
    }
    pub fn inner(&self) -> OodPayload {
        self.inner.clone()
    }
}

impl<S: OodPayloadStreamer> OodPayloadGetter for OodReply<S> {
    type S = S;
    fn get_streamer(&self) -> &Self::S {
        &self.data
    }
    fn get_action(&self) -> &HeaderValue {
        &self.action
    }
    fn get_item(&self) -> &HeaderValue {
        &self.item
    }
}
