use std::net::SocketAddr;

use uuid::Uuid;
use warp::Filter;

use crate::server::{
    OodServer, OodSessionContainer, SessionId,
    handlers::new_session,
    interface::page::{OodPage, OodPageHandler},
};

pub struct EmptyRoute;

pub struct OodServerBuilder<P = EmptyRoute> {
    route: P,
    server_uri: SocketAddr,
    sessions: OodSessionContainer,
}

impl OodServerBuilder {
    pub fn new(server_uri: SocketAddr) -> Self {
        Self {
            server_uri,
            route: EmptyRoute,
            sessions: Default::default(),
        }
    }
}

impl OodServerBuilder<EmptyRoute> {
    pub fn add_route<P>(
        self,
        p: P,
    ) -> OodServerBuilder<
        impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone,
    >
    where
        P: OodPage,
    {
        // let route = p.create_page(&self.sessions);
        let route = new_session_path(p, self.sessions.clone());
        OodServerBuilder {
            route,
            server_uri: self.server_uri,
            sessions: self.sessions,
        }
    }
}

impl<R> OodServerBuilder<R>
where
    R: Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone,
{
    pub fn add_route<P>(
        self,
        p: P,
    ) -> OodServerBuilder<
        impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone,
    >
    where
        P: OodPage,
    {
        // NOTE; we have to use Response which is dynamic because reply can be various things (json, redirect etc)
        OodServerBuilder {
            route: self
                .route
                .or(new_session_path(p, self.sessions.clone()))
                .unify(),
            server_uri: self.server_uri,
            sessions: self.sessions,
        }
    }
}

impl<R> OodServerBuilder<R>
where
    R: Filter<Extract = (warp::reply::Response,), Error = warp::Rejection>
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn start_server(self) -> OodServer {
        OodServer::new(self.route, self.server_uri, self.sessions)
    }
}

pub fn new_session_path<P: OodPage>(
    page: P,
    sessions: OodSessionContainer,
) -> impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone {
    let (page, para_settings) = page.split();
    let para_handler = P::ParaHandler::para_extractor(para_settings);

    para_handler
        .and(warp::head()) // page route will always be a HEAD request
        .and(warp::path::end())
        .and(warp::any().map(move || page.clone()))
        .and(warp::any().map(|| Into::<SessionId>::into(Uuid::new_v4().to_string())))
        .and(warp::any().map(move || sessions.clone()))
        .and_then(new_session::<P::Para, P::PageSession>)
}
