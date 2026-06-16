use std::sync::Arc;

use tokio::{fs, sync::Mutex};

use crate::server::interface::{
    ExternalRedirectType, OodAction,
    elements::{OodButtonList, OodInfo, OodOpenUri},
    page::{OodPageSession, basic::OodBasicPage, para::OodParaPage},
};

#[derive(Clone)]
pub struct Homepage {
    inner: Grocery,
}

impl Homepage {
    pub fn new() -> Self {
        Self {
            inner: Grocery::new(),
        }
    }
}

impl OodBasicPage for Homepage {
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    type SessionPara = ();
    async fn start_session(
        self,
        b: crate::server::interface::bridge::OodBridge,
        _: (),
        _: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        return Ok(b.internal_redirect(self.inner, ()).await);

        loop {
            let options = [
                "Current event",
                "Upcoming events",
                "Past events",
                "Sync with calendar",
                "Settings",
            ];

            match b
                .cf(&OodButtonList::new("Homepage", &options))
                .await?
                .as_str()
            {
                "Settings" => {
                    return Ok(b
                        .external_redirect(ExternalRedirectType::Uri(Settings::URI.into()))
                        .await);
                }
                _ => {
                    b.cf(&OodInfo::new("Unsupported", "")).await?;
                }
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct Grocery {
    todo: Arc<Mutex<Vec<String>>>,
    finished: Arc<Mutex<Vec<String>>>,
}

impl Grocery {
    fn new() -> Self {
        let d = std::fs::read_to_string("grocery.txt")
            .unwrap()
            .split('\n')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Self {
            todo: Arc::new(Mutex::new(d)),
            ..Default::default()
        }
    }
}

impl OodBasicPage for Grocery {
    const URI: &str = "/grocery";
}

impl OodPageSession<()> for Grocery {
    type SessionPara = ();
    async fn start_session(
        self,
        mut b: crate::server::interface::bridge::OodBridge,
        _: (),
        _: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        let mut choice = None;
        loop {
            let Some(sel_choice) = choice.as_ref() else {
                choice = Some(
                    b.cf(&OodButtonList::new("List", &["Todo", "Finished"]))
                        .await?,
                );
                continue;
            };

            match sel_choice.as_ref() {
                "Back" => choice = None,
                "Todo" => {
                    if self.todo.lock().await.len() == 0 {
                        b.cf(&OodInfo::new("Oops", "Nothing in here!")).await?;
                        choice = None;
                        continue;
                    }
                    let mut list = vec!["Back".to_string()];
                    list.extend_from_slice(&self.todo.lock().await.clone()[..]);
                    let el = b.cf(&OodButtonList::new("Todo", &list)).await?;
                    if el == "Back" {
                        choice = None;
                        continue;
                    }
                    println!("Selected {el}");
                    let idx = self
                        .todo
                        .lock()
                        .await
                        .iter()
                        .enumerate()
                        .find_map(|(i, s)| (s == &el).then_some(i))
                        .unwrap();
                    let rem = self.todo.lock().await.remove(idx);
                    self.finished.lock().await.push(rem);
                }
                "Finished" => {
                    if self.finished.lock().await.len() == 0 {
                        b.cf(&OodInfo::new("Oops", "Nothing in here!")).await?;
                        choice = None;
                        continue;
                    }
                    let mut list = vec!["Back".to_string()];
                    list.extend_from_slice(&self.finished.lock().await.clone()[..]);
                    let el = b.cf(&OodButtonList::new("Finished", &list)).await?;
                    if el == "Back" {
                        choice = None;
                        continue;
                    }
                    println!("Selected {el}");
                    let idx = self
                        .finished
                        .lock()
                        .await
                        .iter()
                        .enumerate()
                        .find_map(|(i, s)| (s == &el).then_some(i))
                        .unwrap();
                    let rem = self.finished.lock().await.remove(idx);
                    self.todo.lock().await.push(rem);
                }
                _ => unreachable!(),
            };
        }
        Ok(b.finished().await)
    }
}

#[derive(Clone)]
pub struct Settings;
impl OodBasicPage for Settings {
    const URI: &str = "/settings";
}
impl OodPageSession<()> for Settings {
    type SessionPara = ();
    async fn start_session(
        self,
        mut b: crate::server::interface::bridge::OodBridge,
        _: (),
        _: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        b.cf(&OodInfo::new("Settings", "Welcome to settings"))
            .await?;
        Ok(b.finished().await)
    }
}

// general page to manage an event
#[derive(Clone)]
pub struct EventPage;

impl OodParaPage for EventPage {
    const URI: &str = "/event";
    type Para = String; // event id
}

impl OodPageSession<String> for EventPage {
    type SessionPara = ();
    async fn start_session(
        self,
        b: crate::server::interface::bridge::OodBridge,
        p: String,
        s: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        // managing event `p`

        let options = &["Details", "Add note"];
        Ok(b.finished().await)
    }
}
