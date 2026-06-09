// Credit: https://github.com/ramosbugs/oauth2-rs/blob/main/oauth2/examples/google.rs
// All I did was replace the capture server with warp and enspsulate + split logic into a struct

// WARNING; access token will periodically expire (e.g., ~10hrs for google)

#![allow(dead_code)]

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};
use oauth2::{RefreshToken, reqwest};
use oauth2::{TokenResponse, basic::BasicClient};
use thiserror::Error;

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use warp::Filter;

// just extracted from vscode autocomplete so that I can store
type BasicClientType = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;
type BasicOauthError = oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>;
type BasicTokenType =
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

type OauthTokenError = oauth2::RequestTokenError<
    oauth2::HttpClientError<reqwest::Error>,
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
>;

#[derive(Debug, Error)]
pub enum GoogleAuthError {
    #[error("authorization code was not delivered")]
    AuthCodeNotDelivered,

    #[error("token exchange failed")]
    TokenExchange(#[from] OauthTokenError),

    #[error("response had no refresh token")]
    NoRefreshToken,
}

pub struct GoogleAuth {
    inner: GoogleClient,
    refresh_token: RefreshToken,
}

pub struct GoogleClient {
    scopes: Vec<Scope>,
    redirect_addr: SocketAddr,
    auth_client: BasicClientType,
    http_client: reqwest::Client,
}

impl GoogleClient {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_addr: SocketAddr,
        scopes: Vec<Scope>,
    ) -> Self {
        let google_client_id = ClientId::new(client_id.to_string());
        let google_client_secret = ClientSecret::new(client_secret.to_string());

        let auth_url =
            AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL");

        let client = BasicClient::new(google_client_id)
            .set_client_secret(google_client_secret)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(
                RedirectUrl::new(format!("http://{}", redirect_addr).to_string())
                    .expect("Invalid redirect URL"),
            );

        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        Self {
            auth_client: client,
            http_client,
            redirect_addr,
            scopes,
        }
    }
    pub(crate) async fn capture_authorization_code(
        addr: SocketAddr,
    ) -> Result<(AuthorizationCode, CsrfToken), GoogleAuthError> {
        println!("Capturing code");
        let (code_tx, mut code_rx) = mpsc::channel::<(AuthorizationCode, CsrfToken)>(1);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let redirect = warp::path::end()
            .and(warp::query::<HashMap<String, String>>())
            .map(move |mut query: HashMap<String, String>| {
                let maybe_pair = match (query.remove("code"), query.remove("state")) {
                    (Some(c), Some(s)) => {
                        let code = AuthorizationCode::new(c);
                        let state = CsrfToken::new(s);
                        Some((code, state))
                    }
                    _ => None,
                };

                let tx = code_tx.clone();
                let stx = shutdown_tx.clone();

                if let Some(pair) = maybe_pair {
                    let _ = tx.try_send(pair);
                    warp::reply::html(String::from("Authorised :)"))
                } else {
                    let _ = stx.try_send(());
                    warp::reply::html(String::from("Missing code or state"))
                }
            });

        tokio::spawn(async move {
            let server = warp::serve(redirect);
            let _ = server
                .bind(addr)
                .await
                .graceful(async move {
                    let _ = shutdown_rx.recv().await;
                })
                .run()
                .await;
        });

        let pair = code_rx
            .recv()
            .await
            .ok_or(GoogleAuthError::AuthCodeNotDelivered)?;
        Ok(pair)
    }

    pub(crate) async fn authorize_and_get_refresh_token(
        &self,
    ) -> Result<RefreshToken, GoogleAuthError> {
        // bit inefficient - could store the url, but this is done so infrequently that it really doesn't matter
        let (authorize_url, csrf_state) = self
            .auth_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(self.scopes.clone())
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        println!("Open this URL in your browser:\n{authorize_url}\n");

        let (code, state) = Self::capture_authorization_code(self.redirect_addr).await?;

        assert_eq!(state.secret(), csrf_state.secret());

        let token_response = self
            .auth_client
            .exchange_code(code)
            .request_async(&self.http_client)
            .await?;

        let refresh_token = token_response
            .refresh_token()
            .ok_or(GoogleAuthError::NoRefreshToken)?;

        Ok(refresh_token.clone())
    }

    pub(crate) async fn get_new_access_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<BasicTokenType, GoogleAuthError> {
        Ok(self
            .auth_client
            .exchange_refresh_token(refresh_token)
            .request_async(&self.http_client)
            .await?)
    }
}

impl GoogleAuth {
    pub async fn refresh_access_token(&self) -> Result<BasicTokenType, GoogleAuthError> {
        Ok(self
            .inner
            .get_new_access_token(&self.refresh_token())
            .await?)
    }

    pub fn refresh_token(&self) -> &RefreshToken {
        &self.refresh_token
    }

    fn new(refresh_token: RefreshToken, client: GoogleClient) -> Self {
        Self {
            refresh_token,
            inner: client,
        }
    }

    pub async fn auth_login(client: GoogleClient) -> Result<Self, GoogleAuthError> {
        Ok(Self::new(
            client.authorize_and_get_refresh_token().await?,
            client,
        ))
    }

    pub async fn login(refresh_token: &str, client: GoogleClient) -> Result<Self, GoogleAuthError> {
        Ok(Self::new(
            RefreshToken::new(refresh_token.to_string()),
            client,
        ))
    }
}
