use std::time::Instant;

use restman_rs::ApiHttpClient;

use crate::gcal::{
    GoogleCalendar,
    api::{COLOR_GREEN_ID, COLOR_RED_ID, endpoints::events::EventRes},
};

pub enum CalEventStatus {
    Finished,
    Incomplete,
}
impl From<CalEventStatus> for &str {
    fn from(value: CalEventStatus) -> Self {
        match value {
            CalEventStatus::Finished => COLOR_GREEN_ID,
            CalEventStatus::Incomplete => COLOR_RED_ID,
        }
    }
}

pub struct CalEvent {
    inner: EventRes,
    status: CalEventStatus,
}

// all notifications - point to same url /update which starts current CalEvent
// start logic is: if current CalEvent is already runnign - do nothing
// if some other CalEvent is running - stop that CalEvent then start the current CalEvent

// ---- THIS IS ALL HANDLED IN CurrentCalEvent::end
// CalEvent stop mechanism -> early stop = split and mark the remaining as incomplete
// late stop -> create new CalEvent to indicate excess beyond original plan
// ----

// CalEvent start mechanism - late start
//      - if started inside the original CalEvent, split and mark past section as incomplete
//      - if started outside the original CalEvent, start new CalEvent if all of the past CalEvent is complete
//      - otherwise, try to pull bits of the incomplete CalEvents (COMPLICATED - not necessary)

// early start - create new CalEvent to indicate additional work
// if this early start CalEvent crosses into itself then stop this CalEvent at the other CalEvent's upper bound and continute from there

// if you want to start a missed CalEvent, go to homepage and go through missed CalEvents

// let webhook = warp::path!("webhook" / "gcal")
//     .and(warp::post())
//     .and(refresh_filter)
//     .map(|refresh_tx: mpsc::UnboundedSender<()>| {
//         let _ = refresh_tx.send(());
//         json_response(StatusCode::OK, &serde_json::json!({ "ok": true }))
//     });

// let routes = root.or(today).or(webhook);

// warp::serve(routes).run(self.server_uri).await;
// Self { gcal, bark }

pub struct CurrentCalEvent {
    inner: CalEvent,
    started: Instant,
}

impl CurrentCalEvent {
    fn end<C: ApiHttpClient>(gcal: GoogleCalendar<C>) {
        // ended early? split and mark rest as incomplete
        // ended late? insert new CalEvent to show excess

        // THAT'S IT - we're working off primitives, no need to do anything more fancy
        todo!()
    }
}
