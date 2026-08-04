use std::{convert::Infallible, marker::PhantomData, str::Utf8Error};

use mime::IMAGE_JPEG;
use serde::{
    Deserialize,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::server::interface::{OodParse, OodParseWithContentType, OodPayloadParseError};

#[derive(Debug, Error)]
#[error("not empty")]
pub struct NotEmpty;

impl OodParse for () {
    type E = NotEmpty; // in case user wants to enforce that return is empty (if not, just discard!)
    type O<'a> = ();
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        _: &'a Option<mime::Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        (body.len() == 0).then_some(()).ok_or(NotEmpty.into())
    }
}

impl OodParseWithContentType for str {
    type E = Utf8Error;
    type O<'a>
        = &'a str
    where
        Self: 'a;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a mime::Mime,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        // for now enforce ONLY text - even though things like application/{json, xml etc} may also be valid utf-8/16
        if content_type.type_() != mime::TEXT {
            return Err(OodPayloadParseError::InvalidContentType(
                content_type.clone(),
            ));
        }
        str::from_utf8(body).map_err(Into::into)
    }
}

pub struct OodOptional<T: ?Sized>(PhantomData<T>);

// e.g., Option<str> -> adapter allows for cases where there is no content-type header
impl<T: OodParse + ?Sized> OodParse for OodOptional<T> {
    // I have to remember `+ ?Sized` actually broadens the scope!
    type E = T::E;
    type O<'a> = Option<T::O<'a>>;
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a Option<mime::Mime>,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        if body.trim_ascii().len() == 0 {
            Ok(None)
        } else {
            T::ood_try_from(body, content_type).map(Some)
        }
    }
}

pub struct ImageWrapper(bytes::Bytes);
impl OodParseWithContentType for ImageWrapper {
    type E = Infallible;
    type O<'a> = &'a [u8];
    fn ood_try_from<'a>(
        body: &'a bytes::Bytes,
        content_type: &'a mime::Mime,
    ) -> Result<Self::O<'a>, OodPayloadParseError<Self::E>> {
        if content_type != &IMAGE_JPEG {
            return Err(OodPayloadParseError::InvalidContentType(
                content_type.clone(),
            ));
        }
        Ok(&body[..])
    }
}

// impl<'de, T: Deserialize<'de>> OodParseWithContentType for JsonPayload<'de, T> {
//     type E = serde_json::Error;
//     type O<'a> = T;

//     fn ood_try_from<'a>(
//         body: &'a bytes::Bytes,
//         content_type: &'a mime::Mime,
//     ) -> Result<Self::O<'de>, OodPayloadParseError<Self::E>>
//     where
//         'a: 'de,
//     {
//         // strictly application JSON *only*
//         if content_type != &mime::APPLICATION_JSON {
//             return Err(OodPayloadParseError::InvalidContentType(
//                 content_type.clone(),
//             ));
//         }
//         serde_json::from_slice(body).map_err(Into::into)
//     }
// }