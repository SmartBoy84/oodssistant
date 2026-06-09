use async_trait::async_trait;

use crate::server::{
    OodSessionContainer,
    handlers::new_session,
    interface::page::{OodPagePara, OodPageSession},
};

pub struct OodInternalPayload<S: OodPageSession<P>, P: OodPagePara> {
    s: S,
    p: P,
}

pub trait IntoOodInternalPayload<P>
where
    P: OodPagePara,
    Self: OodPageSession<P>,
{
    fn into_internal_payload(self, p: P) -> OodInternalPayload<Self, P>;
}

impl<P: OodPagePara, S: OodPageSession<P>> IntoOodInternalPayload<P> for S {
    fn into_internal_payload(self, p: P) -> OodInternalPayload<Self, P> {
        OodInternalPayload { s: self, p }
    }
}

// now we create a dyn-compatible trait that will use the payload to spawn the "redirect" task handler
#[async_trait] // async in traits not supported yet - this boxes to make it dyn compatible
pub trait OodInternalRedirect: Send {
    async fn redirect(
        self: Box<Self>,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection>;
}

/*
Remember; IsOneShot and IsSession have different implementations solely because I don't want SessionContainer to be copied uncessarily
for IsOneShot (via the filter)
*/
#[async_trait]
impl<P: OodPagePara + Send, S: OodPageSession<P>> OodInternalRedirect for OodInternalPayload<S, P> {
    async fn redirect(
        self: Box<Self>,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection> {
        /*
        NOTE; this spawns a new sessions rather than carry forward previous session id
        -> each page has its own session id
         */
        let Self { s, p } = *self;
        new_session::<P, S>(p, s, sessions).await
        // she's a'beautiful ma! this took so long to figure out but clean af right?!
    }
}
