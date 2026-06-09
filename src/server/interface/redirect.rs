use std::{marker::PhantomData, pin::Pin};

use async_trait::async_trait;

use crate::server::{OodSessionContainer, handlers::new_session, interface::page::OodPage};

pub struct OodInternalPayload<P: OodPage> {
    s: P::PageSession,
    para: P::Para,
    _page: PhantomData<P>,
}

pub trait IntoOodInternalPayload
where
    Self: OodPage,
{
    fn new_internal_payload(s: Self::PageSession, para: Self::Para) -> OodInternalPayload<Self>;
}

impl<P: OodPage> IntoOodInternalPayload for P {
    fn new_internal_payload(s: P::PageSession, para: P::Para) -> OodInternalPayload<Self> {
        OodInternalPayload {
            s,
            para,
            _page: PhantomData,
        }
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
impl<P: OodPage> OodInternalRedirect for OodInternalPayload<P> {
    fn redirect<'async_trait>(
        self: Box<Self>,
        sessions: OodSessionContainer,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<warp::reply::Response, warp::reject::Rejection>>
                + Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
    {
        /*
        NOTE; this spawns a new sessions rather than carry forward previous session id
        -> each page has its own session id
         */
        Box::pin(new_session::<P>(self.para,self.s, sessions))
        // she's a'beautiful ma! this took so long to figure out but clean af right?!
    }
}
