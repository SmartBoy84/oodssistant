use tokio::sync::mpsc;

use crate::server::interface::{
    OodAction, OodAppErr,
    bridge::{OodBridge, OodFinished},
    elements::{OodOpenUri, OodTextInput},
    page::{IsOneShot, IsSession, OodPageSession, template::OodBasicPage},
};

#[derive(Clone)]
pub struct Homepage {
    pub updated: mpsc::Sender<String>,
}

impl Homepage {
    pub fn new(updated: mpsc::Sender<String>) -> Self {
        Self { updated }
    }
}

impl OodBasicPage for Homepage {
    type PageType = IsOneShot;
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        println!("New homepage!");

        let res = b
            .cf(&OodTextInput::new(
                "What'cha wanna search for, sillay boii?",
                "",
            ))
            .await?;

        b.cf(&OodOpenUri::new(
            "",
            &format!("https://google.com/search?q={res}"),
        ))
        .await?;

        self.updated.send(res).await.unwrap();

        Ok(b.finished().await)
    }
}
