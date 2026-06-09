use async_trait::async_trait;
use tokio::sync::mpsc;
use warp::{Filter, filters::BoxedFilter};

use crate::server::{
    OodSessionContainer,
    interface::{
        OodAppErr, OodReplyType,
        bridge::{OodBridge, OodFinished},
    },
};

pub mod page_type;
pub mod template;

// OodPage types
pub struct IsOneShot;
pub struct IsSession;

// structs containing parameters picked up from custom path handlng
pub trait OodPagePara {}
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
    type PageType: OodPageType;

    type ParaSettings: OodPageParaSettings;
    type PageSession: OodPageSession<Self::Para> + Send + Sync + 'static;
    type ParaHandler: OodPageHandler<Self>;

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
    // remember 'static just means is *can* remain alive for the rest of the program's duration - of course, owner can drop it earlier (not a leak!)
    fn start_session(
        self,
        bridge: OodBridge,
        para: P,
    ) -> impl std::future::Future<Output = Result<OodFinished, OodAppErr>> + Send + 'static;

    fn app_open(
        self,
        para: P,
    ) -> (
        impl Future<Output = Result<OodFinished, OodAppErr>> + Send + 'static,
        mpsc::Receiver<OodReplyType>,
        mpsc::Sender<serde_json::Value>,
    ) {
        let (out_tx, out_rx) = mpsc::channel(1);
        let (in_tx, in_rx) = mpsc::channel(1);
        let fut = self.start_session(OodBridge::new(out_tx, in_rx), para);
        (fut, out_rx, in_tx)
    }
}

// use a similar pattern to my restman-rs library (where POST implements the function)

#[async_trait]
pub trait OodPageType {
    /*
    Don't *fully* get why boxing is necessary here, but the gist of it seems to be:
    if I did impl Filter<...> then implement add_para_handler, because each handler takes an anonymous closure (as well as various other state-dependent things under the hood)
    the return type is varies per call -> but of course trait requires implmentation to yield one concrete type
    Thus, boxing is necessary
     */

    // for external routing (i.e., user visits a page)
    fn add_para_handler<I, P: OodPage>(
        session: P::PageSession,
        i: I,
        sessions: OodSessionContainer,
    ) -> BoxedFilter<(warp::reply::Response,)>
    where
        I: Filter<Extract = (P::Para,), Error = warp::Rejection> + Clone + Send + Sync + 'static;

    // for redirects (internal routing - i.e., one page routes to another page)
    async fn redirect<P: OodPage>(
        session: P::PageSession,
        para: P::Para,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection>;
}

pub trait OodPageHandler<P>
where
    P: OodPage,
    P::PageSession: 'static,
{
    fn create_page(
        p: P,
        sessions: OodSessionContainer,
    ) -> impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone + Send + Sync
    where
        P: Sized,
    {
        let (session, settings) = p.split();
        P::PageType::add_para_handler::<_, P>(session, Self::para_extractor(settings), sessions)
    }

    /// this is the FIRST part of URL (you are responsible for matching URI here)
    fn para_extractor(
        settings: P::ParaSettings,
    ) -> impl Filter<Extract = (P::Para,), Error = warp::Rejection> + Clone + Send + Sync + 'static;
}
