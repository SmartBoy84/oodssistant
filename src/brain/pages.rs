use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use tokio::sync::Mutex;

use crate::server::{
    SessionId,
    interface::{
        OodAction, OodActionHasData, OodActionHasNoData,
        elements::{
            OodButtonList, OodCameraSide, OodInfo, OodMemDelete, OodMemRead, OodMemWrite,
            OodOpenUri, OodStopwatch, OodStopwatchAction, OodTakeImage, OodTextInput, OodTimer,
            Seconds,
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
        // test Json shared Bytes
        b.cf(&OodInfo::new("Hey!", "About to take a photo - ready?")?)
            .await?;
        let image = b.cf(&OodTakeImage::new(&OodCameraSide::Front)?).await?;
        println!("Got it!");
        fs::write("image.jpg", image.p()?).unwrap();
        Ok(b.finished().await)
    }
}
