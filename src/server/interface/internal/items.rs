use std::{borrow::Cow, convert::Infallible, fmt::Display};

use bytes::Bytes;
use serde::Serialize;

use crate::server::interface::internal::TryToOodBytes;

// wrapper to simplify implementation (so that I don't have to implement TryToOodBytes each time) for enume T: Into<'static str>
pub struct OodEnumItem<T: Into<&'static str>>(T);
impl<T: Into<&'static str>> From<T> for OodEnumItem<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T: Into<&'static str>> TryToOodBytes for OodEnumItem<T> {
    type E = Infallible;
    fn to_ood_bytes(self) -> Result<bytes::Bytes, Self::E> {
        Ok(bytes::Bytes::from_static(self.0.into().as_bytes()))
    }
}

// wrapper for T: Display
pub struct OodDisplayItem<T: Display>(T);
impl<T: Display> From<T> for OodDisplayItem<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T: Display> TryToOodBytes for OodDisplayItem<T> {
    type E = Infallible;
    fn to_ood_bytes(self) -> Result<bytes::Bytes, Self::E> {
        Ok(self.0.to_string().into())
    }
}

pub struct JsonItem<T: Serialize>(T);
impl<T: Serialize> TryToOodBytes for JsonItem<T> {
    type E = serde_json::Error;
    fn to_ood_bytes(self) -> Result<Bytes, Self::E> {
        serde_json::to_vec(&self.0).map(Bytes::from)
    }
}

impl<T: Serialize> From<T> for JsonItem<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Into<Cow<'static, str>>> TryToOodBytes for T {
    type E = Infallible;
    fn to_ood_bytes(self) -> Result<bytes::Bytes, Self::E> {
        Ok(match self.into() {
            Cow::Borrowed(value) => Bytes::from_static(value.as_bytes()),
            Cow::Owned(value) => Bytes::from_owner(value),
        })
    }
}

pub enum OodOptionalItem<T: TryToOodBytes> {
    Some(T),
    None,
}
impl<T: TryToOodBytes> From<Option<T>> for OodOptionalItem<T> {
    fn from(value: Option<T>) -> Self {
        value
            .map(OodOptionalItem::Some)
            .unwrap_or(OodOptionalItem::None)
    }
}
impl<T: TryToOodBytes> TryToOodBytes for OodOptionalItem<T> {
    type E = T::E;
    fn to_ood_bytes(self) -> Result<bytes::Bytes, Self::E> {
        Ok(match self {
            Self::None => bytes::Bytes::new(),
            Self::Some(t) => t.to_ood_bytes()?,
        })
    }
}