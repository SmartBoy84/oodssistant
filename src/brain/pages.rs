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
        elements::{
            OodButtonList, OodCameraSide, OodInfo, OodMemDelete, OodMemRead, OodMemWrite,
            OodOpenUri, OodStopwatch, OodStopwatchAction, OodTakeImage, OodTextInput, OodTimer,
        },
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
        b.cf(OodButtonList::new("Choose.", &["1", "2"])?).await?;
        // b.cf(&OodInfo::new("hello", "world")?).await?;
        // Camera: exercise both sides and persist both responses.
        // let front_image = b.cf(&OodTakeImage::new(OodCameraSide::Back)?).await?.p()?;

        // fs::write("ood-camera-front.jpg", front_image.as_ref()).unwrap();

        // let back_image = b.cf(&OodTakeImage::new(OodCameraSide::Back)?).await?.p()?;

        // fs::write("ood-camera-back.jpg", back_image.as_ref()).unwrap();

        // Write a persistent value.
        // b.cf(&OodMemWrite::new("asdasd", "tet")?)
        //     .await?
        //     .p()?;

        // // Read it back.
        // let stored_value = b.cf(&OodMemRead::new("test.txt")?).await?.p()?;
        // println!("read: {stored_value:?}");
        // //         let stored_value = b.cf(&OodMemRead::new("test.txt")?).await?.p()?;
        // // println!("read: {stored_value:?}");

        // // Delete it.
        // b.cf(&OodMemDelete::new("test.txt")?).await?.p()?;

        // // Open an external URI.
        // b.cf(&OodOpenUri::new("https://example.com")?).await?.p()?;

        // // Display information with an attached data value.
        // b.cf(&OodInfo::new(
        //     "Test information",
        //     "Information supplied by the server",
        // )?)
        // .await?
        // .p()?;

        // // Display a list of buttons.
        // //
        // // This specifically exercises JsonItemWrap<[T]> with a borrowed,
        // // dynamically sized slice.
        // let buttons = [
        //     String::from("First"),
        //     String::from("Second"),
        //     String::from("Third"),
        // ];

        // let selected_button: String = b
        //     .cf(&OodButtonList::new("Choose a button", &buttons)?)
        //     .await?
        //     .p()?;

        // // Start a timer.
        // b.cf(&OodTimer::new(Some(Duration::from_secs(30).into()))?)
        //     .await?
        //     .p()?;

        // // Deactivate the timer.
        // b.cf(&OodTimer::new(None)?).await?.p()?;

        // // Exercise every stopwatch action.
        // let started: String = b
        //     .cf(&OodStopwatch::new(OodStopwatchAction::Start)?)
        //     .await?
        //     .p()?;

        // let stopped: String = b
        //     .cf(&OodStopwatch::new(OodStopwatchAction::Stop)?)
        //     .await?
        //     .p()?;

        // let reset: String = b
        //     .cf(&OodStopwatch::new(OodStopwatchAction::Reset)?)
        //     .await?
        //     .p()?;

        // // Text input with a default value and prompt data.
        // let entered_text = b
        //     .cf(&OodTextInput::new("Default value", "Enter some text")?)
        //     .await?
        //     .p()?;
        // println!("{entered_text:?}");

        Ok(b.finished().await)
    }
}
