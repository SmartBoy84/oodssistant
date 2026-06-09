use tokio::sync::mpsc;
use warp::Filter;

use crate::server::{
    OodSessionContainer,
    handlers::SessionId,
    interface::{
        OodAppErr, OodReplyType,
        bridge::{OodBridge, OodFinished},
    },
};

pub mod basic;
pub mod para;

// for structs being populated in new handler
pub trait IsOodSessionPara {
    fn new(session_id: &SessionId, sessions: &OodSessionContainer) -> Self;
}

impl IsOodSessionPara for () {
    // this is for clients that do not need session parameters (avoid redundant copying)
    fn new(_: &SessionId, _: &OodSessionContainer) {
        ()
    }
}

// goes WITHOUT saying, do not try to search for yourself in sessions using session_id
pub struct OodSessionPara {
    pub session_id: SessionId,
    pub sessions: OodSessionContainer,
}
impl IsOodSessionPara for OodSessionPara {
    fn new(session_id: &SessionId, sessions: &OodSessionContainer) -> Self {
        // make a copy of everything so that it is accessible inside the task
        Self {
            session_id: session_id.to_owned(),
            sessions: sessions.to_owned(),
        }
    }
}

// structs containing parameters picked up from custom path handlng
pub trait OodPagePara: Send {}
impl OodPagePara for () {}

// optimisation - so that I don't have to clone the whole thing!
pub trait OodPageParaSettings {} // contains everything to passed to its para handler
impl OodPageParaSettings for &'static str {}

// start here

// for custom handlers (to obtain OodPagePara)
pub trait OodPage: Send
where
    Self: Sized,
{
    type ParaSettings: OodPageParaSettings;
    type PageSession: OodPageSession<Self::Para> + 'static;
    type ParaHandler: OodPageHandler<Self::ParaSettings, Self::Para>;

    type Para: OodPagePara + Send;

    // encode generics on the function because these are fixed for a given type
    fn split(self) -> (Self::PageSession, Self::ParaSettings);
}

// OodPageSession *MUST* be Cloneable so that multiple sessions can be spawned
// this means any environment variables must be wrapped in an Arc<Mutex<T>>
// Send requirement is pretty obvious - we are running in a concurrent environment

// thus, only support dynamic URLs to faciliate page "flavours" ('static) -> don't support url generation using environment state (runtime)
// I do this because `Uri` can be generated from borrowed strings

// note; can't impose ParaHandler trait here as that would create a cycle!
pub trait OodPageSession<P: OodPagePara>: Clone + Send {
    type SessionPara: IsOodSessionPara;

    // remember 'static just means is *can* remain alive for the rest of the program's duration - of course, owner can drop it earlier (not a leak!)
    fn start_session(
        self,
        b: OodBridge,
        p: P,
        s: Self::SessionPara,
    ) -> impl std::future::Future<Output = Result<OodFinished, OodAppErr>> + Send + 'static;

    fn app_open(
        self,
        p: P,
        s: Self::SessionPara,
    ) -> (
        impl Future<Output = Result<OodFinished, OodAppErr>> + Send + 'static,
        mpsc::Receiver<OodReplyType>,
        mpsc::Sender<serde_json::Value>,
    ) {
        let (out_tx, out_rx) = mpsc::channel(1);
        let (in_tx, in_rx) = mpsc::channel(1);
        let fut = self.start_session(OodBridge::new(out_tx, in_rx), p, s);
        (fut, out_rx, in_tx)
    }
}

// use a similar pattern to my restman-rs library (where POST implements the function)

pub trait OodPageHandler<S: OodPageParaSettings, P: OodPagePara> {
    /// WARNING; add a warp::get() (or whatever you want) BUT dont add a end()

    /// this is the FIRST part of URL (you are responsible for matching URI here)
    fn para_extractor(
        settings: S,
    ) -> impl Filter<Extract = (P,), Error = warp::Rejection> + Clone + Send;
}
