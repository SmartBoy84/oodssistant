use std::{
    collections::HashMap, error::Error, fmt::Display, net::SocketAddr, ops::Deref, sync::Arc,
    time::Duration,
};

use crate::server::{
    handlers::{get_session_cache, session_handler},
    interface::{ExtOodAppErr, OodReplyType, external::OodResponse},
    request::{OodPayload, OodStandardReq},
};
use mime::Mime;
use thiserror::Error;
use tokio::{
    sync::{Mutex, mpsc},
    task::{JoinError, JoinHandle},
    time::Instant,
};

use warp::Filter;

pub mod builder;
pub mod handlers;
pub mod interface;
pub mod request;

// run janitor every minute
const CLEANUP_PERIOD: Duration = Duration::from_secs(60);
const TASK_EXPIRY: Duration = Duration::from_secs(30); // tasks expire after 30 seconds

const JSON_MAX_LENGTH: u64 = 1024 * 16;

const SESSION_PART: &str = "session";
const ITEM_HEADER: &str = "ood-item";
const ACTION_HEADER: &str = "ood-action";
const ID_HEADER: &str = "ood-id";

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct SessionId(Arc<String>);

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self(Arc::new(value))
    }
}

impl Deref for SessionId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default, Clone)]
pub struct OodSessionContainer {
    inner: Arc<Mutex<HashMap<SessionId, OodSession>>>,
}
impl OodSessionContainer {
    pub async fn evict_session(&self, s: &SessionId) {
        if let Some(s) = self.inner.lock().await.remove(s) {
            s.end();
        }
    }
}

impl Deref for OodSessionContainer {
    type Target = Arc<Mutex<HashMap<SessionId, OodSession>>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct OodServer {
    sessions: OodSessionContainer,
    janitor: Option<JoinHandle<()>>,
    server: JoinHandle<()>,
}

pub struct OodSession {
    rx: mpsc::Receiver<OodReplyType>,
    tx: mpsc::Sender<Result<OodResponse, Box<dyn Error + Sync + Send>>>,
    last_payload: Option<OodPayload>,
    last_change: Instant,
    task: JoinHandle<()>,
}

impl OodSession {
    fn end(self) {
        self.task.abort();
    }
}

#[derive(Error, Debug, Clone)]
enum OodReqErr {
    #[error("session task crashed")]
    SessionTaskEnded,

    #[error("session not found")]
    SessionNotFound,

    #[error("backend err")]
    BackendErr(ExtOodAppErr),

    #[error("bad redirect uri")]
    BadRedirectUri,

    #[error("cache is empty")]
    EmptyCache,
}

impl warp::reject::Reject for OodReqErr {}

async fn janitor_task(sessions: OodSessionContainer, run_period: Duration, expiry: Duration) {
    loop {
        tokio::time::sleep(run_period).await;

        let now = Instant::now();

        let mut sessions_guard = sessions.lock().await;
        sessions_guard.retain(|id, session| {
            if now.duration_since(session.last_change) > expiry {
                session.task.abort(); // since I do a bunch of .unwrap() I don't want it panicking because I dropped channel
                println!("janitor: evicted session {id}");
                false
            } else {
                true
            }
        })
    }
}

impl OodSession {
    async fn send(
        &self,
        p: Result<OodResponse, Box<dyn Error + Sync + Send>>,
    ) -> Result<(), warp::Rejection> {
        self.tx
            .send(p)
            .await
            .map_err(|_| warp::reject::custom(OodReqErr::SessionTaskEnded))
    }

    async fn recv(&mut self) -> Result<OodReplyType, warp::Rejection> {
        self.rx
            .recv()
            .await
            .ok_or(warp::reject::custom(OodReqErr::SessionTaskEnded))
    }
}

impl OodServer {
    fn new<P>(p: P, server_uri: SocketAddr, sessions: OodSessionContainer) -> Self
    where
        P: Filter<Extract = (warp::reply::Response,), Error = warp::Rejection>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let session_filter = {
            let sessions = sessions.clone();

            // extract session id
            let session_base = warp::path(SESSION_PART)
                .and(warp::path::param::<String>())
                .map(Into::into) // don't reply on warp parsing because that is faillible (unncessary)
                .and(warp::path::end())
                .and(warp::any().map(move || sessions.clone()));

            // present cache
            let get_session_route = session_base
                .clone()
                .and(warp::get())
                .and_then(get_session_cache::<OodStandardReq>); // as per my communication flow, this is the only way to get actual data

            // HEAD request
            let head_session_route = session_base
                .clone()
                .and(warp::head())
                .and_then(|s_id, s_c| session_handler(s_id, s_c, None, None));

            // drive forwards
            let post_session_route = session_base
                .and(warp::post())
                .and(warp::body::bytes().map(Some))
                .and(warp::header::optional::<Mime>("content-type"))
                .and_then(session_handler);

            get_session_route
                .or(post_session_route)
                .or(head_session_route)
        };

        let server_path = session_filter.or(p).with(warp::log::custom(|info| {
            if info.status() == warp::http::StatusCode::NOT_FOUND {
                println!("unknown path: {} ({})", info.path(), info.method())
            }
            println!(
                "[request] remote={:?} method={} path={} status={} elapsed={}ms",
                info.remote_addr(),
                info.method(),
                info.path(),
                info.status(),
                info.elapsed().as_millis(),
            );
        }));

        let server = tokio::spawn(warp::serve(server_path).run(server_uri));

        println!("Started server on {server_uri:?}");

        let mut ood = Self {
            sessions: sessions.clone(),
            janitor: None,
            server,
        };

        let _ = ood.spawn_janitor(CLEANUP_PERIOD, TASK_EXPIRY);
        ood
    }

    /// returns None if it was already running
    pub async fn spawn_janitor(&mut self, run_period: Duration, expiry: Duration) -> Option<()> {
        if let Some(_) = self.janitor {
            return None;
        }

        let janitor = tokio::spawn(janitor_task(self.sessions.clone(), run_period, expiry));

        // cannot just drop old janitor as it will continue running in background (JoinHandle doesn't have special drop logic)
        self.janitor = Some(janitor);
        Some(())
    }

    #[allow(unused)]
    pub async fn stop_janitor(&mut self) -> Result<(), JoinError> {
        let Some(janitor) = self.janitor.take() else {
            return Ok(());
        };
        janitor.abort();
        janitor.await
    }

    // WARNING; DO NOT DROP THIS FUTURE ELSE THE TASK ENDS!
    pub async fn await_server(self) -> Result<(), JoinError> {
        self.server.await
    }
}
