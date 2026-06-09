use crate::gcal::{
    api::error::GCalApiError,
    client::GCalClient,
    login::{GoogleAuth, GoogleAuthError, GoogleClient},
};
use oauth2::{Scope, TokenResponse};
use restman_rs::{
    ApiBackendError, ApiHttpClient, ConstServer, Server, backends::reqwest::ReqwestApiHttpClient,
    client::AGENT,
};
use std::net::SocketAddr;
use thiserror::Error;

pub mod api;
mod client;
pub mod frontend;
mod login; // doesn't need to be public

const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/calendar"];
const GCAL_ROOT: &str = "https://www.googleapis.com";

#[derive(Debug, Error)]
pub enum GCalErr<C: ApiHttpClient> {
    #[error("auth error")]
    GoogleAuthError(#[from] GoogleAuthError),

    #[error("api error")]
    ApiBackendErr(#[from] ApiBackendError<C>),

    #[error(transparent)]
    GoogleApiError(#[from] GCalApiError),

    #[error("calendar not found")]
    CalendarNotFound,

    #[error("payload parse error")]
    PayloadParseError(#[from] serde_json::Error)
}

pub struct GoogleCalendar<C: ApiHttpClient> {
    auth: GoogleAuth,
    backend: GCalClient<C>,
}

pub struct GCalServer;
impl Server for GCalServer {}
impl ConstServer for GCalServer {
    const ROOT: &str = GCAL_ROOT;
}

pub struct GoogleCalendarBuilder<T> {
    inner: GoogleClient,
    backend: T,
}

impl GoogleCalendar<ReqwestApiHttpClient> {
    pub fn builder(
        client_id: &str,
        client_secret: &str,
        redirect_addr: SocketAddr,
    ) -> GoogleCalendarBuilder<ReqwestApiHttpClient> {
        let inner = GoogleClient::new(
            client_id,
            client_secret,
            redirect_addr,
            SCOPES.map(|s| Scope::new(s.to_string())).to_vec(),
        );
        GoogleCalendarBuilder::<ReqwestApiHttpClient>::new(inner)
    }
}

impl<C: ApiHttpClient> GoogleCalendar<C> {
    pub fn builder_with_client(
        client_id: &str,
        client_secret: &str,
        redirect_addr: SocketAddr,
        backend: C,
    ) -> GoogleCalendarBuilder<C> {
        let inner = GoogleClient::new(
            client_id,
            client_secret,
            redirect_addr,
            SCOPES.map(|s| Scope::new(s.to_string())).to_vec(),
        );
        GoogleCalendarBuilder::<C>::new_with_client(inner, backend)
    }

    pub fn backend(&mut self) -> &mut GCalClient<C> {
        &mut self.backend
    }

    fn auth(&mut self) -> &mut GoogleAuth {
        &mut self.auth
    }

    async fn refresh_token(&mut self) -> Result<(), GCalErr<C>> {
        let token = self.auth().refresh_access_token().await?;
        self.backend().update_token(token.access_token().secret());
        Ok(())
    }
}

impl<T> GoogleCalendarBuilder<T> {
    fn new(inner: GoogleClient) -> GoogleCalendarBuilder<ReqwestApiHttpClient> {
        let backend = ReqwestApiHttpClient::new(AGENT);
        GoogleCalendarBuilder { inner, backend }
    }

    fn new_with_client<C: ApiHttpClient>(
        inner: GoogleClient,
        backend: C,
    ) -> GoogleCalendarBuilder<C> {
        GoogleCalendarBuilder { inner, backend }
    }
}

impl<C: ApiHttpClient> GoogleCalendarBuilder<C> {
    pub async fn new_login(self) -> Result<GoogleCalendar<C>, GCalErr<C>> {
        let Self { inner, backend } = self;
        let auth = GoogleAuth::auth_login(inner).await?;
        println!("Refresh token: {}", auth.refresh_token().secret());

        Ok(Self::build(backend, auth).await?)
    }

    pub async fn login(self, refresh_token: &str) -> Result<GoogleCalendar<C>, GCalErr<C>> {
        let Self { inner, backend } = self;
        let auth = GoogleAuth::login(refresh_token, inner).await?;
        Ok(Self::build(backend, auth).await?)
    }

    async fn build(backend: C, auth: GoogleAuth) -> Result<GoogleCalendar<C>, GCalErr<C>> {
        let mut cal = GoogleCalendar {
            backend: GCalClient::new(backend),
            auth,
        };
        cal.refresh_token().await?; // must refresh at the start to actually obtain the access token!
        Ok(cal)
    }
}
