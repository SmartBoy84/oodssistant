use std::{convert::Infallible, marker::PhantomData, string::FromUtf8Error};

use mime::IMAGE_JPEG;
use thiserror::Error;

use crate::server::interface::{OodParse, OodPayloadParseError, external::OodParseWithContentType};

#[derive(Debug, Error)]
#[error("not empty")]
pub struct NotEmpty;

impl OodParse for () {
    type E = NotEmpty; // in case user wants to enforce that return is empty (if not, just discard!)
    type O = ();
    fn ood_try_from(
        body: bytes::Bytes,
        _: Option<mime::Mime>,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>> {
        (body.len() == 0).then_some(()).ok_or(NotEmpty.into())
    }
}

impl OodParseWithContentType for String {
    type E = FromUtf8Error;
    type O = String;

    fn ood_try_from(
        body: bytes::Bytes,
        content_type: mime::Mime,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>> {
        // for now enforce ONLY text - even though things like application/{json, xml etc} may also be valid utf-8/16
        if content_type.type_() != mime::TEXT {
            return Err(OodPayloadParseError::InvalidContentType(
                content_type.clone(),
            ));
        }
        String::from_utf8(body.into()).map_err(Into::into)
    }
}

pub struct OodOptional<T>(PhantomData<T>);

// e.g., Option<str> -> adapter allows for cases where there is no content-type header
impl<T: OodParse> OodParse for OodOptional<T> {
    // I have to remember `+ ?Sized` actually broadens the scope!
    type E = T::E;
    type O = Option<T::O>;
    fn ood_try_from(
        body: bytes::Bytes,
        content_type: Option<mime::Mime>,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>> {
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
    type O = bytes::Bytes; // do not need to convert to Vec<u8>
    fn ood_try_from(
        body: bytes::Bytes,
        content_type: mime::Mime,
    ) -> Result<Self::O, OodPayloadParseError<Self::E>> {
        if content_type != IMAGE_JPEG {
            return Err(OodPayloadParseError::InvalidContentType(
                content_type.clone(),
            ));
        }
        Ok(body)
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
