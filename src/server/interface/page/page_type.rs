use std::pin::Pin;

use uuid::Uuid;
use warp::{Filter, filters::BoxedFilter, reply::Reply};

use crate::server::{
    OodSessionContainer,
    handlers::new_session,
    interface::page::{IsOneShot, IsSession, OodPage, OodPageType},
};

impl OodPageType for IsSession {
    fn add_para_handler<I, P: OodPage>(
        session: P::PageSession,
        i: I,
        sessions: OodSessionContainer,
    ) -> BoxedFilter<(warp::reply::Response,)>
    where
        I: Filter<Extract = (P::Para,), Error = warp::Rejection> + Clone + Send + Sync + 'static,
    {
        i.and_then(move |para: P::Para| {
            Self::new_session::<P>(session.clone(), para, sessions.clone())
        })
        .boxed()
    }

    fn redirect<'async_trait, P>(
        session: P::PageSession,
        para: P::Para,
        sessions: OodSessionContainer,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<warp::reply::Response, warp::reject::Rejection>>
                + Send
                + 'async_trait,
        >,
    >
    where
        P: 'async_trait + OodPage,
    {
        Box::pin(Self::new_session::<P>(session, para, sessions))
    }
}

impl OodPageType for IsOneShot {
    fn add_para_handler<I, P: OodPage>(
        session: P::PageSession,
        i: I,
        sessions: OodSessionContainer,
    ) -> BoxedFilter<(warp::reply::Response,)>
    where
        I: Filter<Extract = (P::Para,), Error = warp::Rejection> + Clone + Send + Sync + 'static,
    {
        i.and_then(move |para: P::Para| {
            Self::new_session::<P>(session.clone(), para, sessions.clone())
        })
        .boxed()
    }

    fn redirect<'async_trait, P>(
        session: P::PageSession,
        para: P::Para,
        sessions: OodSessionContainer,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<warp::reply::Response, warp::reject::Rejection>>
                + Send
                + 'async_trait,
        >,
    >
    where
        P: 'async_trait + OodPage,
    {
        Box::pin(Self::new_session::<P>(session, para, sessions))
    }
}

// implement new_session function separately so that I do not uncessarily close OodSessionContainer
impl IsSession {
    pub async fn new_session<P: OodPage>(
        session: P::PageSession,
        para: P::Para,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection> {
        let page = session.clone();
        let session_id = Uuid::new_v4().to_string();

        let (session, reply) =
            new_session::<P>(page, para, Some(session_id.clone()), &sessions).await?;

        let _ = sessions.lock().await.insert(session_id, session); // make persistent
        Ok::<_, warp::Rejection>(reply.into_response())
    }
}

impl IsOneShot {
    pub async fn new_session<P: OodPage>(
        session: P::PageSession,
        para: P::Para,
        sessions: OodSessionContainer,
    ) -> Result<warp::reply::Response, warp::reject::Rejection> {
        let page = session.clone();

        let (_, reply) = new_session::<P>(page, para, None, &sessions).await?;

        Ok::<_, warp::Rejection>(reply.into_response())
    }
}
