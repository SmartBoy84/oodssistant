use std::{borrow::Cow, convert::Infallible, fmt::Display, marker::PhantomData};

use bytes::Bytes;
use serde::Serialize;

use crate::server::interface::internal::TryToOodBytes;

// json types
pub struct JsonSlice<'a, T: ?Sized>(&'a T);
impl<'a, T: Serialize + ?Sized, U: AsRef<T> + ?Sized> From<&'a U> for JsonSlice<'a, T> {
    fn from(value: &'a U) -> Self {
        Self(value.as_ref())
    }
}

// generic json wrapper
pub struct JsonItem<T: Serialize + ?Sized>(PhantomData<fn(&T)>);
impl<S: Serialize + ?Sized> TryToOodBytes for JsonItem<S> {
    type E = serde_json::Error;
    type O<'a>
        = JsonSlice<'a, S>
    where
        S: 'a;
    fn to_ood_bytes<'a>(s: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a,
    {
        serde_json::to_vec(s.0).map(Bytes::from)
    }
}

// parser to simplify implementation (so that I don't have to implement TryToOodBytes each time) for enume T: Into<'static str>
pub struct OodEnumItem<T: Into<&'static str>>(PhantomData<fn(&T)>);

impl<T: Into<&'static str>> TryToOodBytes for OodEnumItem<T> {
    type E = Infallible;
    type O<'a>
        = T
    where
        T: 'a;
    fn to_ood_bytes<'a>(s: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a,
    {
        Ok(bytes::Bytes::from_static(s.into().as_bytes()))
    }
}

// wrapper for T: Display
pub struct OodDisplayItem<T: Display>(PhantomData<fn(&T)>);

impl<T: Display> TryToOodBytes for OodDisplayItem<T> {
    type E = Infallible;
    type O<'a>
        = T
    where
        Self: 'a;
    fn to_ood_bytes<'a>(s: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a,
    {
        Ok(s.to_string().into())
    }
}

impl TryToOodBytes for Cow<'static, str> {
    type E = Infallible;
    type O<'a> = Self;
    fn to_ood_bytes<'a>(o: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a,
    {
        Ok(match o {
            Cow::Borrowed(value) => Bytes::from_static(value.as_bytes()),
            Cow::Owned(value) => Bytes::from_owner(value),
        })
    }
}

// a bit confusing but this is not a wrapper - it is a "parser", as well
pub struct OodOptionalItem<T: TryToOodBytes>(PhantomData<fn(&T)>);

impl<T: TryToOodBytes> TryToOodBytes for OodOptionalItem<T> {
    type E = T::E;
    type O<'a>
        = Option<T::O<'a>>
    where
        T: 'a;
    fn to_ood_bytes<'a>(s: Self::O<'a>) -> Result<bytes::Bytes, Self::E>
    where
        Self: 'a,
    {
        Ok(match s {
            None => bytes::Bytes::new(),
            Some(t) => T::to_ood_bytes(t)?,
        })
    }
}
