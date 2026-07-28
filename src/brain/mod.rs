use restman_rs::request::ApiPayload;
use thiserror::Error;
use tokio::task::JoinError;

use crate::{
    bark::{
        BarkClient, BarkError,
        payload::{BarkPayload, PushLevel},
    },
    brain::{calendar::OodCalErr, pages::Homepage},
    gcal::GoogleCalendar,
    server::{OodServer, builder::OodServerBuilder, interface::page::basic::OodStatic},
};

pub mod calendar;
pub mod pages;
pub mod shortcut;

const OOD_ERROR_NOTIF_GROUP: &str = "ood_error";

// The Ood, at long last!
pub struct Ood {
    bark: BarkClient,
    // cal: OodCalendar,
    server: OodServer,
}

#[derive(Debug, Error)]
pub enum OodErr {
    #[error(transparent)]
    CalErr(#[from] OodCalErr),

    #[error(transparent)]
    BarkErr(#[from] BarkError),
}

type OodResult<T> = Result<T, OodErr>;

impl Ood {
    pub async fn new(
        gcal: GoogleCalendar,
        bark: BarkClient,
        server_builder: OodServerBuilder,
        calendar_name: &str,
    ) -> OodResult<Self> {
        let server = server_builder
            .add_route(OodStatic(Homepage::default()))
            .start_server();

        // let cal = OodCalendar::build_new(gcal, calendar_name).await?;
        // Ok(Self { cal, bark, server })

        Ok(Self { bark, server })
    }

    async fn send_error(&self, err: impl Into<String>) -> OodResult<()> {
        self.bark
            .notify(
                &ApiPayload::new(
                    &BarkPayload::builder()
                        .body(err.into())
                        .level(PushLevel::Active)
                        .group(OOD_ERROR_NOTIF_GROUP)
                        .build(),
                )
                .map_err(|e| OodErr::BarkErr(BarkError::SerdeError(e)))?,
            )
            .await?;
        Ok(())
    }

    pub async fn run_me(self) -> Result<(), JoinError> {
        self.server.await_server().await
    }
}
