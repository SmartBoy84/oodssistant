use restman_rs::{
    ApiHttpClient,
    client::{ApiClientBackend, ApiClientServer},
};

use crate::bark::api::BarkServer;

pub struct BarkClientInner<C: ApiHttpClient> {
    backend: C,
}

impl<C: ApiHttpClient> From<C> for BarkClientInner<C> {
    fn from(value: C) -> Self {
        Self { backend: value }
    }
}

impl<C: ApiHttpClient> ApiClientBackend<C> for BarkClientInner<C> {
    fn backend(&self) -> &C {
        &self.backend
    }
}

impl<C: ApiHttpClient> ApiClientServer<BarkServer> for BarkClientInner<C> {}
