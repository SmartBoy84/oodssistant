use std::{collections::HashMap, str::FromStr};

use chrono::{Days, Local, TimeZone, Utc};
use strum::{Display, EnumProperty, EnumString, VariantNames};
use thiserror::Error;

use crate::gcal::{
    GCalErr, GoogleCalendar,
    api::{
        EventColour,
        endpoints::{
            calendar_list::CalendarRes,
            events::{EventRes, OrderBy},
        },
    },
};

#[derive(Debug, Error)]
pub enum OodCalErr {
    #[error("event has no end time")]
    NoEndTime((String, String)),
    #[error("event has no start time")]
    NoStartTime((String, String)),
    #[error("transparent")]
    GCalErr(#[from] GCalErr),
}

struct OodEvent {
    id: String, // included as an immutable marker for events
    name: String,
    description: String,
    start_time: chrono::DateTime<Utc>,
    end_time: chrono::DateTime<Utc>,
    status: OodEventStatus,
}

/*
 it will try to tag whichever events it can
 events in the past that are not tagged are left untagged
 events in the past that are tagged inconsistently are updated accordingly
 events in the future that are tagged are left unchanged
*/

const EVENT_STATUS_TAG: &str = "%%"; // surround name with %% to identify

#[derive(Display, VariantNames, EnumString, EnumProperty, PartialEq, Hash, Eq)]
pub enum OodEventStatus {
    Upcoming,   // upcoming events
    Finished,   // past + finished
    Incomplete, // past + not finished
    Unknown,    // past + not tagged
}

#[derive(Default)]
pub struct OodCalTheme(HashMap<OodEventStatus, EventColour>);

pub struct OodCalendar {
    gcal: GoogleCalendar, // no playing around anymore - use their default clients (reqwest)
    cal_id: String,
    theme: OodCalTheme,
    events: HashMap<String, OodEvent>, // (Id, Event)
}

/*
Initial idea: get_upcoming() -> add an alarm for each event -> fires notifications independently
*/

impl OodCalendar {
    pub async fn build_new(mut gcal: GoogleCalendar, cal_name: &str) -> Result<Self, OodCalErr> {
        let CalendarRes { id, .. } = gcal.find_calendar(cal_name).await?;
        let mut cal = Self {
            cal_id: id,
            gcal,
            theme: OodCalTheme::default(),
            events: HashMap::new(),
        };
        cal.calendar_sync().await?;
        Ok(cal)
    }

    async fn update_color(&mut self, event_id: &str, color: EventColour) -> Result<(), OodCalErr> {
        todo!()
    }

    // called infrequently
    async fn calendar_sync(&mut self) -> Result<(), OodCalErr> {
        // midnight -> midnight
        let today = Local::now().date_naive();

        let start_local = Local
            .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();

        let end_local = start_local.checked_add_days(Days::new(1)).unwrap();

        let events_res = self
            .gcal
            .get_events(
                &self.cal_id,
                start_local.with_timezone(&Utc),
                end_local.with_timezone(&Utc),
                OrderBy::StartTime,
            )
            .await?;

        // regenerate the hashmap
        self.events.drain().for_each(drop); // drain the map

        for EventRes {
            id,
            summary,
            description,
            start,
            color,
            end,
            ..
        } in events_res
        {
            let description = description.unwrap_or("".to_string());
            let summary = summary.unwrap_or("".to_string());

            let Some(ref start_time) = start else {
                return Err(OodCalErr::NoStartTime((summary, description)));
            };
            let Some(ref end_time) = end else {
                return Err(OodCalErr::NoEndTime((summary, description)));
            };
            if start_time
                .timezone
                .as_deref()
                .or(end_time.timezone.as_deref())
                != None
            {
                panic!("AHH, who dee hell r u? I can't handle time zones brudda");
            }

            let status = description.find("%%").and_then(|start| {
                if start + 2 >= description.len() - 1 {
                    return None;
                }
                let Some(end) = description[start + 2..].find("%%").map(|i| i + start + 2) else {
                    return None;
                };
                let tag = &description[start..end];
                println!("found tag: {tag}");
                OodEventStatus::from_str(tag).ok()
            });

            if let Some(ref status) = status
                && let Some(expected_colour) = self.theme.0.get(&status)
                && expected_colour != &color
            {
                println!("Mismatching colour - updating");
                self.update_color(&id, expected_colour.clone()).await?;
            }

            self.events.insert(
                id.clone(),
                OodEvent {
                    id,
                    name: summary,
                    description,
                    start_time: start_time.date_time,
                    end_time: end_time.date_time,
                    status: status.unwrap_or(OodEventStatus::Unknown),
                },
            );
        }

        Ok(())
    }
}
