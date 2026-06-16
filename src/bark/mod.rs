use restman_rs::{
    APost, ApiBackendError, ApiHttpClient, backends::reqwest::ReqwestApiHttpClient, client::async_client::ApiClient, request::{ApiPayload, ApiRequest}
};
use thiserror::Error;

use crate::bark::{
    api::{BarkRequest, BarkRes},
    client::BarkClientInner,
    payload::BarkPayload,
};

pub mod api;
pub mod builder;
pub mod client;
pub mod payload;

#[derive(Debug, Error)]
pub enum BarkError<C: ApiHttpClient = ReqwestApiHttpClient> {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    ApiBackendError(#[from] ApiBackendError<C>),

    #[error("bark api error: code={}, message={}", .0.code, .0.message)]
    BarkApiError(#[allow(unused)] BarkRes),
}

// Obtain using BarkClientBuilder
pub struct BarkClient<C: ApiHttpClient = ReqwestApiHttpClient> {
    inner: BarkClientInner<C>,
    req: ApiRequest<BarkRequest>,
}

impl<C: ApiHttpClient + Sync> BarkClient<C> {
    fn inner(&self) -> &BarkClientInner<C> {
        &self.inner
    }

    pub async fn notify(&self, c: &ApiPayload<BarkPayload>) -> Result<BarkRes, BarkError<C>>
    where
        C: APost,
    {
        let res: Result<_, _> = self.inner().async_send_payload(&self.req, c).await?.into();
        Ok(res.map_err(|res| BarkError::BarkApiError(res))?) // a bit weird, but eh
    }
}
