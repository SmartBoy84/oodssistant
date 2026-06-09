/*
Yes I tried my BEST to minimise container arc cloning - the idea was I'd pass a reference around and when it need to mutate it
it would clone it but the lifetimes became hellish to manage (mentally and in practise)
-> especially when I had to use #[async_trait] so I opted to just clone everywhere!
*/

use reqwest::header::CONTENT_TYPE;
use serde::Serialize;
use tokio::time::Instant;
use uuid::Uuid;
use warp::reply::Reply;

use crate::server::{
    OodReqErr, OodSession, OodSessionContainer,
    interface::{
        OodReplyType,
        page::{OodPagePara, OodPageSession},
    },
};

// NOTE; session_id is NON-negotiable (I tried other options, trust me... like one shot page)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OodSessionPayload {
    session_id: String,
    payload: serde_json::Value,
}

impl OodSessionPayload {
    fn new(session_id: String, payload: serde_json::Value) -> Self {
        Self {
            payload,
            session_id,
        }
    }
}

/*
new_session needs sessions because new_session -> session_handler -> Redirect -> IsSession -> needs to append!
*/
// it is independent of OodPage to support "internal pages"
pub async fn new_session<P: OodPagePara, S: OodPageSession<P>>(
    p: P,
    s: S,
    sessions: OodSessionContainer,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    let session_id = Uuid::new_v4().to_string();

    let (fut, out_rx, in_tx) = s.app_open(p);

    let task = tokio::spawn(async {
        if let Err(e) = fut.await {
            println!("{e:?}")
        }
    });

    let session = OodSession {
        rx: out_rx,
        tx: in_tx,
        task,
        last_payload: None,
        last_change: Instant::now(),
    };

    let _ = sessions.lock().await.insert(session_id.clone(), session); // make persistent
    let first_res = session_handler(session_id, sessions, None).await;

    Ok(first_res?)
}

pub fn make_json_response(payload: String) -> warp::reply::Response {
    let mut res = warp::reply::Response::new(payload.into());

    // copied from warp::reply::json(..).into_response()
    res.headers_mut().insert(
        CONTENT_TYPE,
        warp::http::HeaderValue::from_static("application/json"),
    );
    return res;
}

pub async fn get_session_cache(
    session_id: String,
    sessions: OodSessionContainer,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    println!("Cache request");

    let mut session_guard = sessions.lock().await;
    let session = session_guard
        .get_mut(&session_id)
        .ok_or(OodReqErr::SessionNotFound)?;

    match &session.last_payload {
        Some(cached_payload) => Ok(make_json_response(cached_payload.to_string())),
        None => Err(warp::reject::custom(OodReqErr::EmptyCache)),
    }
}

pub async fn session_handler(
    session_id: String,
    sessions: OodSessionContainer,
    body: Option<serde_json::Value>,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    let mut session_guard = sessions.lock().await;

    // if coming from new_session, this is a bit redundant but it is the best way to avoid deadlock that I could think of
    let session = session_guard
        .get_mut(&session_id)
        .ok_or(OodReqErr::SessionNotFound)?;

    println!("comm [{session_id}]");

    if let Some(body) = body {
        session.send(body).await? // if not, we are in an initial request
    } else {
        session.last_change = Instant::now(); // i.e., last time this endpoint was queried
    }

    let res = session.recv().await?;

    match res {
        OodReplyType::Payload(p) => {
            println!("sending: {p}");
            let payload = serde_json::to_string(&OodSessionPayload::new(session_id, p))
                .map_err(|e| OodReqErr::SerialisationError(e))?;
            session.last_payload = Some(payload.clone());
            return Ok(make_json_response(payload));
        }

        // don't need to set last_payload = None in the following because for all of these the page function must have returned OodFinished (task has ended)
        OodReplyType::Redirect(u) => {
            println!("requested redir");
            drop(session_guard); // V IMPORTANT! Else will dead-lock
            Ok(u.redirect(sessions).await?) // hallelujah - so, so, so much effort is underlying this simple thing!
        }
        OodReplyType::Finished => return Ok(warp::reply().into_response()),
        OodReplyType::Error(e) => return Err(warp::reject::custom(OodReqErr::BackendErr(e))),
    }
}
