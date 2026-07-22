use std::{
    str::FromStr,
    sync::Arc,
    time::{self, Duration},
};

use strum::VariantNames;
use tokio::{fs, sync::Mutex, time::Instant};

use crate::server::interface::{
    OodAction, OodActionHasNoSummary, OodActionHasSummary,
    elements::{
        OodButtonList, OodInfo, OodOpenUri, OodStopwatch, OodStopwatchAction, OodTextInput,
        OodTimer, Seconds,
    },
    page::{OodPageSession, basic::OodBasicPage, para::OodParaPage},
};

#[derive(Clone, Default)]
pub struct Homepage {
    pallets: Arc<Mutex<Vec<(String, time::Duration)>>>,
    curr_name: Arc<Mutex<Option<String>>>,
    current: Arc<Mutex<Option<tokio::time::Duration>>>,
    running: Arc<Mutex<Option<tokio::time::Instant>>>,
}

fn format_duration(d: &tokio::time::Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn option_dur_add(running: Option<Instant>, past_total: Option<Duration>) -> Duration {
    let curr_total = running.map(|t| tokio::time::Instant::now() - t);
    match (curr_total, past_total) {
        (Some(t), None) | (None, Some(t)) => t,
        (Some(a), Some(b)) => a + b,
        _ => unreachable!(), // finish would not be visible otherwise
    }
}

impl OodBasicPage for Homepage {
    const URI: &str = "/";
}

impl OodPageSession<()> for Homepage {
    type SessionPara = ();
    async fn start_session(
        self,
        mut b: crate::server::interface::bridge::OodBridge,
        _: (),
        _: (),
    ) -> Result<crate::server::interface::bridge::OodFinished, crate::server::interface::OodAppErr>
    {
        loop {
            let mut options = vec![];
            let mut menu_title = None;
            if let Some(name) = self.curr_name.lock().await.as_ref().map(|s| s.clone()) {
                let total = option_dur_add(*self.running.lock().await, *self.current.lock().await);
                menu_title = Some(format!("{name} - {}", format_duration(&total)));
                if *&self.running.lock().await.is_some() {
                    options.push("Pause");
                    options.push("Finish");
                } else if self.current.lock().await.is_some() {
                    options.push("Resume");
                    options.push("Finish")
                }
            } else {
                options.push("New")
            }
            if self.pallets.lock().await.len() > 0 {
                options.push("Past pallets");
            }
            match b
                .cf(&OodButtonList::new(
                    menu_title.as_deref().unwrap_or("Menu"),
                    &options,
                ))
                .await?
                .as_str()
            {
                "Past pallets" => {
                    b.cf(&OodInfo::new(
                        "Past pallets",
                        &self
                            .pallets
                            .lock()
                            .await
                            .iter()
                            .map(|(n, t)| format!("{n} - {}", format_duration(t)))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ))
                    .await?;
                }
                "Pause" => {
                    let dur = tokio::time::Instant::now()
                        - self.running.lock().await.take().expect("not running?");
                    let mut guard = self.current.lock().await;
                    if let Some(t) = guard.as_mut() {
                        *t += dur;
                    } else {
                        *guard = Some(dur);
                    }
                    b.cf(&OodStopwatch::new(&OodStopwatchAction::Stop)).await?;
                    b.cf(&OodInfo::new(
                        &format!(
                            "Paused: {}",
                            self.curr_name
                                .lock()
                                .await
                                .as_ref()
                                .expect("no name?")
                                .clone()
                        ),
                        "",
                    ))
                    .await?;
                }
                "Finish" => {
                    let total = option_dur_add(
                        self.running.lock().await.take(),
                        self.current.lock().await.take(),
                    );
                    let name = self.curr_name.lock().await.take().expect("no name?");
                    self.pallets.lock().await.push((name.clone(), total));

                    b.cf(&OodInfo::new(
                        &format!("Ended: {name}"),
                        &format!("Took: {}", format_duration(&total)),
                    ))
                    .await?;
                    b.cf(&OodStopwatch::new(&OodStopwatchAction::Reset)).await?;
                }
                "New" => {
                    let name = b.cf(&OodTextInput::new("Pallet name?", "")).await?;
                    match b
                        .cf(&OodButtonList::new(
                            &format!("Start {name}?"),
                            &["Yes", "No"],
                        ))
                        .await?
                        .as_str()
                    {
                        "Yes" => {
                            *self.curr_name.lock().await = Some(name.clone());
                            *self.running.lock().await = Some(tokio::time::Instant::now());
                            b.cf(&OodStopwatch::new(&OodStopwatchAction::Reset)).await?;
                            b.cf(&OodStopwatch::new(&OodStopwatchAction::Start)).await?;
                            b.cf(&OodInfo::new(&format!("Started: {name}"), "")).await?;
                        }
                        "No" => continue,
                        _ => unreachable!(),
                    };
                }
                "Resume" => {
                    *self.running.lock().await = Some(tokio::time::Instant::now());
                    b.cf(&OodStopwatch::new(&OodStopwatchAction::Start)).await?;
                }
                _ => unreachable!(),
            }
            break; // break by default
        }
        Ok(b.finished().await)
    }
}
