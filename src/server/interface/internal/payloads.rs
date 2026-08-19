use std::{convert::Infallible, marker::PhantomData};

use bytes::Bytes;
use futures_util::{future, stream};

use crate::server::request::OodPayloadStreamer;

impl OodPayloadStreamer for () {
    type StreamErr = Infallible;
    type E = Infallible;
    type B = Bytes;
    type S = stream::Empty<Result<Bytes, Infallible>>;
    fn get_data(
        &self,
    ) -> Result<<Self as OodPayloadStreamer>::S, <Self as OodPayloadStreamer>::StreamErr> {
        Ok(stream::empty())
    }
    fn len(&self) -> Option<usize> {
        None
    }
    fn validate(&self) -> Result<(), <Self as OodPayloadStreamer>::StreamErr> {
        Ok(())
    }
}

pub struct SharedBytes<T: ?Sized> {
    inner: Bytes,
    p: PhantomData<fn(&T)>, // use fn(&T) to make it not owning (-> T can be not Send but MemStreamer is)
}
impl<T: ?Sized> Clone for SharedBytes<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            p: PhantomData,
        }
    }
}
impl<T: ?Sized, U> From<U> for SharedBytes<T>
where
    U: Into<bytes::Bytes> + AsRef<T>, // WARNING; + AsRef<T> is NEEDED to statically enforce that you create SharedBytes from correct origin type
{
    fn from(value: U) -> Self {
        Self {
            // NOTE; 'static is respected by the Into implementations of Bytes
            inner: value.into(),
            p: PhantomData,
        }
    }
}

impl<T: ?Sized + 'static> OodPayloadStreamer for SharedBytes<T> {
    type StreamErr = Infallible;
    type E = Infallible;
    type B = Bytes;
    type S = stream::Once<future::Ready<Result<Bytes, Infallible>>>;

    fn get_data(
        &self,
    ) -> Result<<Self as OodPayloadStreamer>::S, <Self as OodPayloadStreamer>::StreamErr> {
        Ok(stream::once(future::ready(Ok(self.inner.clone()))))
    }
    fn len(&self) -> Option<usize> {
        Some(self.inner.len())
    }
    fn validate(&self) -> Result<(), <Self as OodPayloadStreamer>::StreamErr> {
        Ok(())
    }
}
