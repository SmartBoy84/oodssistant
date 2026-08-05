/*
Yes I tried my BEST to minimise container arc cloning - the idea was I'd pass a reference around and when it need to mutate it
it would clone it but the lifetimes became hellish to manage (mentally and in practise)
-> especially when I had to use #[async_trait] so I opted to just clone everywhere!
*/

use std::str::FromStr;

use mime::Mime;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use tokio::time::Instant;
use warp::reply::Reply;

use crate::server::{
    OodPayload, OodReqErr, OodSession, OodSessionContainer, SessionId, interface::{
        ExtOodAppErr, OodReplyType, external::OodResponse, page::{IsOodSessionPara, OodPagePara, OodPageSession},
    }, request::{OodHeaderReq, OodReqMaker},
};

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OodClientPayload {
    #[serde_as(as = "DisplayFromStr")]
    session_id: SessionId,
    payload: serde_json::Value,
}

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

pub async fn make_response<T: OodReqMaker>(
    p: OodPayload,
    session: &mut OodSession,
    id: &SessionId,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    match T::make(&p, id) {
        Ok(r) => {
            session.last_payload = Some(p);
            Ok(r)
        }
        Err(e) => {
            // pretty nifty - due to my step-by-step comm, downcast here is OK
            let ext_e = OodReqErr::BackendErr(ExtOodAppErr::InternalParseError(e.to_string()));
            session.send(Err(e)).await?;

            // 1. cache presentation path:
            //      we are going to report back that cache presentation failed - backend expects to be able to act on this
            //      the bridge will exit and the receiver must fill again => this is CORRECT
            // 2. regular path
            //      client has already sent the message - if an error occurs then the message stream has progressed so the client must be "stepped" forwards
            session.last_payload = None;
            Err(ext_e.into())
        }
    }
}

pub async fn get_session_cache<T: OodReqMaker>(
    session_id: SessionId,
    sessions: OodSessionContainer,
) -> Result<warp::reply::Response, warp::reject::Rejection> {
    println!("Cache request");

    let mut session_guard = sessions.lock().await;
    let session = session_guard
        .get_mut(&session_id)
        .ok_or(OodReqErr::SessionNotFound)?;

    let r = session.last_payload.take().ok_or(OodReqErr::EmptyCache)?;
    make_response::<T>(r, session, &session_id).await
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
        session.send(Ok(OodResponse { body, content_type })).await? // if not, we are in an initial request
    }

    session.last_change = Instant::now(); // i.e., last time this endpoint was queried
    let res = session.recv().await?;

    match res {
        // communication flow is get -> post -> headers
        // session_handler is only called when there is a post request, initial request or HEAD request
        OodReplyType::Payload(payload) => {
            make_response::<OodHeaderReq>(payload, session, &session_id).await
        }

        // don't need to set last_payload = None in the following because for all of these the page function must have returned OodFinished (task has ended)
        OodReplyType::Finished => Ok(warp::reply().into_response()),
        OodReplyType::InternalRedirect(s_id) => {
            drop(session_guard); // V IMPORTANT! Else will dead-lock
            Ok(get_session_cache::<OodHeaderReq>(s_id, sessions).await?)
        }
        OodReplyType::ExternalRedirect(u) => Ok(warp::redirect::see_other(
            warp::http::Uri::from_str(&u).expect("bad external redir url?"),
        )
        .into_response()),
    }
}
