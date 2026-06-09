use tokio::sync::mpsc;

use crate::server::interface::{
    OodAction, OodAppErr,
    bridge::{OodBridge, OodFinished},
    elements::{OodInfo, OodTextInput},
    page::{OodPageSession, template::OodBasicPage},
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
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        let Self { updated } = self;
        println!("New homepage!");

        let res = b
            .cf(&OodTextInput::new(
                "What'cha wanna search for, sillay boii?",
                "",
            ))
            .await?;

        b.cf(&OodInfo::new("", &format!("Ok, I'll search for {res}")))
            .await?;

        // b.cf(&OodOpenUri::new(
        //     "",
        //     &format!("https://google.com/search?q={res}"),
        // ))
        // .await?;

        updated.send(res).await.unwrap();

        Ok(b.redirect(Homepage::new(updated), ()).await)
    }
}

#[derive(Clone)]
pub struct Subpage1;
impl OodBasicPage for Subpage1 {
    const URI: &str = "/oogabooga";
}
impl OodPageSession<()> for Subpage1 {
    async fn start_session(self, mut b: OodBridge, _: ()) -> Result<OodFinished, OodAppErr> {
        b.cf(&OodInfo::new("hello", "world")).await?;
        Ok(b.finished().await)
    }
}
