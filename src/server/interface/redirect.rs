use async_trait::async_trait;

use crate::server::{
    OodSessionContainer, SessionId,
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
        s_id: SessionId,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection>;
}

#[async_trait]
impl<P: OodPagePara + Send, S: OodPageSession<P>> OodInternalRedirect for OodInternalPayload<S, P> {
    /*
    This seems extremely pointless - after all, why not just use a simple function call?
    This exists solely to support redirects to pages (i.e., pages that can be visited as well)
     */
    async fn redirect(
        self: Box<Self>,
        s_id: SessionId,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection> {
        /*
        NOTE; this spawns a new session but uses the SAME session id
        */
        let Self { s, p } = *self;
        new_session(p, s, s_id, sessions).await
        // she's a'beautiful ma! this took so long to figure out but clean af right?!
    }
}
