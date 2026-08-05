use std::{borrow::Cow, convert::Infallible, marker::PhantomData};

use oauth2::http::HeaderValue;
use reqwest::header::InvalidHeaderValue;
use serde::Serialize;
use warp::reply::Json;

use crate::server::interface::{
    elements::{OodCameraSide, OodStopwatchAction},
    internal::ToOodItemHeader,
};

pub struct OodItem<T> {
    h: HeaderValue,
    _t: PhantomData<fn(&T)>,
}

impl<T> OodItem<T> {
    fn new(h: HeaderValue) -> Self {
        Self { h, _t: PhantomData }
    }
}

impl<T> From<OodItem<T>> for HeaderValue {
    fn from(value: OodItem<T>) -> Self {
        value.h
    }
}

pub struct JsonItem {
    inner: String,
}
impl<T: Serialize> TryFrom<T> for JsonItem {
    type Error = serde_json::Error;
    fn try_from(value: T) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: serde_json::to_string(&value)?,
        })
    }
}

impl<T: Serialize> ToOodItemHeader for JsonItem<T> {
    type IntoErr = serde_json::Error;
    fn to_header(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::try_from(self.inner)
    }
}

impl ToOodItemHeader for &'static str {
    type IntoErr = Infallible;
    fn to_header(self) -> Result<HeaderValue, InvalidHeaderValue> {
        Ok(HeaderValue::from_static(self))
    }
}
impl ToOodItemHeader for Cow<'static, str> {
    type IntoErr = Infallible;
    fn to_header(self) -> Result<HeaderValue, InvalidHeaderValue> {
        let v = match self {
            Self::Borrowed(b) => HeaderValue::from_static(b),
            Cow::Owned(b) => HeaderValue::try_from(b)?,
        };
        Ok(v)
    }
}

impl ToOodItemHeader for OodCameraSide {
    type IntoErr = Infallible;
    fn to_header(self) -> Result<HeaderValue, InvalidHeaderValue> {
        Ok(HeaderValue::from_static(self.into()))
    }
}

impl ToOodItemHeader for OodStopwatchAction {
    type IntoErr = Infallible;
    fn to_header(self) -> Result<HeaderValue, InvalidHeaderValue> {
        Ok(HeaderValue::from_static(self.into()))
    }
}
