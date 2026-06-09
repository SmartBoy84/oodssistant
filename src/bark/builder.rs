#![allow(dead_code)]

use std::borrow::Cow;

// ahhh, don't look - not proud of this... very very very pointless optimisations here
use restman_rs::{
    APost, ApiHttpClient, backends::reqwest::ReqwestApiHttpClient, client::AGENT,
    request::ApiRequest,
};

use crate::bark::{
    BarkClient,
    api::{BarkConfig, BarkRequest, BarkServer},
};

#[derive(Default)]
pub struct Unset {}

#[derive(Default)]
pub struct BarkClientBuilder<'a, B = Unset, S = Unset> {
    backend: B,
    server: S,
    key: Cow<'a, str>,
}

impl<'a> BarkClientBuilder<'a> {
    pub fn new(key: &'a str) -> Self {
        Self {
            key: Cow::Borrowed(key),
            ..Default::default()
        }
    }
}

// option setters
impl<'a, B> BarkClientBuilder<'a, B, Unset> {
    pub fn server(self, server: BarkServer) -> BarkClientBuilder<'a, B, BarkServer> {
        BarkClientBuilder {
            backend: self.backend,
            server,
            key: self.key,
        }
    }
}

impl<'a, S> BarkClientBuilder<'a, Unset, S> {
    pub fn backend<C: ApiHttpClient + APost>(self, backend: C) -> BarkClientBuilder<'a, C, S> {
        BarkClientBuilder {
            backend,
            server: self.server,
            key: self.key,
        }
    }
}

// possible builders
/*
I know this seems like it wouldn't scale - it won't! Everything other than builder can be turned into an option and separated into a different function
But I thought may as well do the static route since only have two config paras
*/

// ugh, after implementing do lowkey feel like I went a bit overboard here...

impl<'a, C: ApiHttpClient + APost> BarkClientBuilder<'a, C, BarkServer> {
    pub fn build(self) -> BarkClient<C> {
        let BarkClientBuilder {
            backend,
            server,
            key,
        } = self;
        BarkClient {
            inner: backend.into(),
            req: ApiRequest::<BarkRequest>::new_with_server(&BarkConfig { key }, &server),
        }
    }
}

impl<'a, C: ApiHttpClient + APost> BarkClientBuilder<'a, C, Unset> {
    pub fn build(self) -> BarkClient<C> {
        let BarkClientBuilder { backend, key, .. } = self;
        BarkClient {
            inner: backend.into(),
            req: ApiRequest::<BarkRequest>::new(&BarkConfig { key }),
        }
    }
}

impl<'a> BarkClientBuilder<'a, Unset, BarkServer> {
    pub fn build(self) -> BarkClient<ReqwestApiHttpClient> {
        let BarkClientBuilder { server, key, .. } = self;
        BarkClient {
            inner: ReqwestApiHttpClient::new(AGENT).into(),
            req: ApiRequest::<BarkRequest>::new_with_server(&BarkConfig { key }, &server),
        }
    }
}

impl<'a> BarkClientBuilder<'a, Unset, Unset> {
    pub fn build(self) -> BarkClient<ReqwestApiHttpClient> {
        let BarkClientBuilder { key, .. } = self;
        BarkClient {
            inner: ReqwestApiHttpClient::new(AGENT).into(),
            req: ApiRequest::<BarkRequest>::new(&BarkConfig { key }),
        }
    }
}
