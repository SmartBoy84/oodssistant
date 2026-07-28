use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::server::{
    SessionId,
    interface::{
        OodAction, OodActionHasNoSummary, OodActionHasSummary,
        elements::{
            OodButtonList, OodInfo, OodMemRead, OodMemWrite, OodOpenUri, OodStopwatch,
            OodStopwatchAction, OodTextInput, OodTimer, Seconds,
        },
        page::{OodPageSession, OodSessionPara, basic::OodBasicPage, para::OodParaPage},
    },
};

const DEVICE_ID: &str = ".OOD_ID"; // .into() is FREE - this is relative to a selected root (on the client side)

#[derive(Clone, Default)]
pub struct Homepage {
    conns: Arc<Mutex<HashSet<SessionId>>>,
}

impl OodBasicPage for Homepage {
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    type SessionPara = OodSessionPara;
    async fn start_session(
        self,
        mut b: crate::server::interface::bridge::OodBridge,
        _: (),
        OodSessionPara { session_id }: Self::SessionPara,
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::ExtOodAppErr>
    {
        let device_id = match b.cf(&OodMemRead::new(DEVICE_ID.into())).await?.p().await? {
            "" => {
                b.cf(&OodInfo::new("Id not found", "")).await?;
                b.cf(&OodMemWrite::new(DEVICE_ID.into(), &session_id))
                    .await?;
                session_id
            }
            id => id.to_owned().into(),
        };
        if self.conns.lock().await.contains(&device_id) {
            b.cf(&OodInfo::new("Restoring old session", "")).await?;
            return Ok(b.internal_redirect(&device_id).await);
        }

        // create new session
        let _ = *&self.conns.lock().await.insert(device_id);
        for i in 0..100 {
            b.cf(&OodInfo::new(&format!("{i}"), "")).await?;
        }
        Ok(b.finished().await)
    }
}
