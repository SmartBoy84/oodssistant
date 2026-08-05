/*
Model

Headers:
ood-action: OodAction::Name - action type (e.g., save file, delete file, take picture etc)
ood-item: OodAction::Item - primary action subject (e.g., filename, alert title)

response data: auxilliary data - (e.g., alert description, file binary data etc)

*/

use std::sync::Arc;

use oauth2::http::HeaderValue;
use reqwest::header::CONTENT_LENGTH;
use warp::reply::{Reply, Response};

use crate::server::{ACTION_HEADER, ID_HEADER, ITEM_HEADER, SessionId};

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type OodPayload = Arc<dyn OodPayloadResponder>;

pub trait OodPayloadResponder: Send + Sync {
    fn make_response(&self, id: &SessionId) -> GenericResult<warp::reply::Response>;
    fn make_head_response(&self, id: &SessionId) -> GenericResult<warp::reply::Response>; // shortcuts needs this, but also makes sense because next step may be like uploading a massive ass file which the user might not have space for (e.g.)
}

pub trait OodReqMaker {
    fn make(p: &OodPayload, id: &SessionId) -> GenericResult<warp::reply::Response>;
}
pub struct OodHeaderReq;
impl OodReqMaker for OodHeaderReq {
    fn make(p: &OodPayload, id: &SessionId) -> GenericResult<warp::reply::Response> {
        p.make_head_response(id)
    }
}
pub struct OodStandardReq;
impl OodReqMaker for OodStandardReq {
    fn make(p: &OodPayload, id: &SessionId) -> GenericResult<warp::reply::Response> {
        p.make_response(id)
    }
}

pub trait OodPayloadStreamer: Send + Sync + 'static {
    type StreamErr: std::error::Error + Send + Sync; // "outer" error - currently only this reaches the backend
    // in the future, if I need I can work on getting ...:E back there as well

    // streaming para
    type E: Into<Box<dyn std::error::Error + Send + Sync>> + Send + 'static; // e.g., failed to open file etc
    type B: Into<bytes::Bytes>;
    type S: futures_util::Stream<Item = Result<Self::B, Self::E>> + Send + Sync + 'static;

    fn validate(&self) -> Result<(), <Self as OodPayloadStreamer>::StreamErr>; // validate without acting (e.g., check if file still exists)
    fn get_data(
        &self,
    ) -> Result<<Self as OodPayloadStreamer>::S, <Self as OodPayloadStreamer>::StreamErr>;
    fn len(&self) -> Option<usize>; // make None specification explicit
}

pub trait OodPayloadGetter: Send + Sync {
    type S: OodPayloadStreamer;

    fn get_item(&self) -> &HeaderValue; // creating a HeaderValue requires parsing and copying &str but this is done on this trait user's side
    fn get_action(&self) -> &HeaderValue;
    fn get_streamer(&self) -> &Self::S;

    fn apply_headers(&self, res: &mut warp::reply::Response, id: &SessionId) {
        if let Some(len) = self.get_streamer().len() {
            res.headers_mut().insert(CONTENT_LENGTH, len.into());
        }

        // couldn't find an easy way to cache so we clone instead
        res.headers_mut()
            .insert(ACTION_HEADER, self.get_action().clone());
        println!("{:?}", self.get_item().to_str().unwrap());
        res.headers_mut()
            .insert(ITEM_HEADER, self.get_item().clone());
        res.headers_mut().insert(
            ID_HEADER,
            HeaderValue::from_str(&id).expect("non-ascii session id?"),
        );
    }
}

/*
WARNING; don't do anything falliable here
*/
impl<T: OodPayloadGetter> OodPayloadResponder for T {
    fn make_response(&self, id: &SessionId) -> GenericResult<Response> {
        let res = match self.get_streamer().get_data() {
            Err(e) => {
                return Err(Box::new(e));
            }

            // no easy way to get streaming errors out unfortunately - eh whatever
            Ok(r) => warp::reply::stream(r),
        }
        .into_response();

        // self.apply_headers(&mut res); // implementation detail - do not need to set these on this request

        // TODO; content-type?

        Ok(res)
    }

    fn make_head_response(&self, id: &SessionId) -> GenericResult<warp::reply::Response> {
        self.get_streamer().validate()?; // validate

        let mut response = warp::reply().into_response();
        self.apply_headers(&mut response, id);
        Ok(response)
    }
}
