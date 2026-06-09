use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;
use warp::{Filter, filters::BoxedFilter};

use crate::server::{
    OodServer, OodSessionContainer,
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
            sessions: Arc::new(Mutex::new(HashMap::new())),
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
        P: OodPage + 'static,
    {
        // let route = p.create_page(&self.sessions);
        let route = P::ParaHandler::create_page(p, self.sessions.clone());
        OodServerBuilder {
            route,
            server_uri: self.server_uri,
            sessions: self.sessions,
        }
    }
}

impl OodServerBuilder<BoxedFilter<(warp::reply::Response,)>> {
    pub fn add_route<P>(
        self,
        p: P,
    ) -> OodServerBuilder<
        impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone,
    >
    where
        P: OodPage + 'static,
    {
        // NOTE; we have to use Response which is dynamic because reply can be various things (json, redirect etc)
        OodServerBuilder {
            route: self
                .route
                .or(P::ParaHandler::create_page(p, self.sessions.clone()).boxed())
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
