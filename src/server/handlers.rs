/*
Yes I tried my BEST to minimise container arc cloning - the idea was I'd pass a reference around and when it need to mutate it
it would clone it but the lifetimes became hellish to manage (mentally and in practise)
-> especially when I had to use #[async_trait] so I opted to just clone everywhere!
*/

use std::str::FromStr;

use mime::Mime;
use reqwest::header::CONTENT_TYPE;
use tokio::time::Instant;
use warp::reply::Reply;

use crate::server::{
    OodReqErr, OodSession, OodSessionContainer, SessionId,
    interface::{
        OodPayload, OodReplyType,
        page::{IsOodSessionPara, OodPagePara, OodPageSession},
    },
};

/*
new_session needs sessions because new_session -> session_handler -> Redirect -> IsSession -> needs to append!
*/
// it is independent of OodPage to support "internal pages"
pub async fn new_session<P: OodPagePara, S: OodPageSession<P>>(
    p: P,
    s: S,
    s_id: SessionId,
    sessions: OodSessionContainer,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    let (fut, out_rx, in_tx) = s.app_open(p, S::SessionPara::new(&s_id));

    let task = {
        let s_id = s_id.clone();
        let sessions = sessions.clone();
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                println!("{e:?}")
            }
            println!("Task ended - removing");
            sessions.lock().await.remove(&s_id); // task finished, remove itself
        })
    };

    let session = OodSession {
        rx: out_rx,
        tx: in_tx,
        task,
        last_payload: None,
        last_change: Instant::now(),
    };

    let _ = sessions.lock().await.insert(s_id.clone(), session); // make persistent
    let first_res = session_handler(s_id, sessions, None, None).await;

    Ok(first_res?)
}

pub fn make_json_response(payload: bytes::Bytes) -> warp::reply::Response {
    let mut res = warp::reply::Response::new(payload.into());

    // copied from warp::reply::json(..).into_response()
    res.headers_mut().insert(
        CONTENT_TYPE,
        warp::http::HeaderValue::from_static("application/json"),
    );
    return res;
}

pub async fn get_session_cache(
    session_id: SessionId,
    sessions: OodSessionContainer,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    println!("Cache request");

    let mut session_guard = sessions.lock().await;
    let session = session_guard
        .get_mut(&session_id)
        .ok_or(OodReqErr::SessionNotFound)?;

    match &session.last_payload {
        Some(cached_payload) => Ok(make_json_response(cached_payload.clone())),
        None => Err(warp::reject::custom(OodReqErr::EmptyCache)),
    }
}

pub async fn session_handler(
    session_id: SessionId,
    sessions: OodSessionContainer,
    body: Option<bytes::Bytes>,
    content_type: Option<Mime>,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    let mut session_guard = sessions.lock().await;

    // if coming from new_session, this is a bit redundant but it is the best way to avoid deadlock that I could think of
    let session = session_guard
        .get_mut(&session_id)
        .ok_or(OodReqErr::SessionNotFound)?;

    println!("comm [{session_id}]");

    if let Some(body) = body {
        session.send(OodPayload { body, content_type }).await? // if not, we are in an initial request
    } else {
        session.last_change = Instant::now(); // i.e., last time this endpoint was queried
    }

    let res = session.recv().await?;

    match res {
        OodReplyType::Payload(p) => {
            session.last_payload = Some(p.clone());
            return Ok(make_json_response(p));
        }

        // don't need to set last_payload = None in the following because for all of these the page function must have returned OodFinished (task has ended)
        OodReplyType::Finished => Ok(warp::reply().into_response()),
        OodReplyType::Err(e) => Err(warp::reject::custom(OodReqErr::BackendErr(e))),
        OodReplyType::InternalRedirect(s_id) => {
            drop(session_guard); // V IMPORTANT! Else will dead-lock
            Ok(get_session_cache(s_id, sessions).await?)
            // match r {
            //     InternalRedirectType::NewPage(u) => Ok(u.redirect(session_id, sessions).await?), // hallelujah - so, so, so much effort is underlying this simple thing!
            //     InternalRedirectType::Session(s_id) => Ok(get_session_cache(s_id, sessions).await?),
            // }
        }
        OodReplyType::ExternalRedirect(u) => Ok(warp::redirect::see_other(
            warp::http::Uri::from_str(&u).expect("bad external redir url?"),
        )
        .into_response()),
    }
}
