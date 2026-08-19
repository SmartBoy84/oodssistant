use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    ops::Deref,
    path::Path,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};

use bytes::Bytes;
use tokio::sync::Mutex;

use crate::server::{
    SessionId,
    interface::{
        elements::{OodButtonList, OodInfo, OodStopwatch, OodStopwatchAction, OodTimer},
        internal::{OodActionHasData, OodActionHasNoData, payloads::SharedBytes},
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
        let s = SharedBytes::from(String::from("123"));
        b.cf(&OodInfo::new(s, "123")?).await?;
        b.cf(&OodStopwatch::new(OodStopwatchAction::Reset)?).await?;
        b.cf(&OodTimer::new(Some(Duration::from_secs(1).into()))?)
            .await?;
        b.cf(&OodButtonList::new("test", &["1"])?).await?;

        Ok(b.finished().await)
    }
}
