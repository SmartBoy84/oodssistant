use restman_rs::{
    ApiHttpClient,
    client::{ApiClientBackend, ApiClientServer},
};

use crate::gcal::GCalServer;

pub struct GCalClient<C: ApiHttpClient> {
    c: C,
}

impl<C: ApiHttpClient> GCalClient<C> {
    pub fn new(c: C) -> Self {
        Self { c }
    }

    pub fn update_token(&mut self, token: &str) {
        self.c.set_bearer_token(token);
    }
}

impl<C: ApiHttpClient> ApiClientBackend<C> for GCalClient<C> {
    fn backend(&self) -> &C {
        &self.c
    }
}

impl<C: ApiHttpClient> ApiClientServer<GCalServer> for GCalClient<C> {}
