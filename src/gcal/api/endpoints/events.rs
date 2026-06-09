use bon::Builder;
use chrono::Utc;
use restman_rs::{
    DELETE, GET, POST, PUT, endpoint,
    request::{QueryParameters, QueryPayload},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::gcal::{
    GCalServer,
    api::{
        GCalApiRes,
        endpoints::{CalDateTime, ListRes},
        request::{EventsPart, EventsPartWithId},
    },
};

// used a lot of Option<> here to be conservative and avoid crashing
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventRes {
    // again cherry picked the ones I *need*
    pub etag: String,
    pub id: String,
    pub summary: Option<String>, // i.e., name
    pub description: Option<String>,
    pub start: Option<CalDateTime>,
    pub end: Option<CalDateTime>,
    pub end_time_unspecified: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum OrderBy {
    StartTime,
    Updated, // by in ascending order, by modification time
}

#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct EventsListGetPara {
    // time range is actually optional, but in the context of my app it is necessary
    time_max: Option<chrono::DateTime<Utc>>,
    time_min: Option<chrono::DateTime<Utc>>,
    // #[builder(default = 2500)] // max is 2500 as per docs
    max_result: Option<usize>,
    #[builder(default = OrderBy::StartTime)]
    // default is a unspecified, but stable order - I would prefer start time
    order_by: OrderBy, // several others but these were the important ones (refer to https://developers.google.com/workspace/calendar/api/v3/reference/calendarList/list)
    page_token: Option<String>, // for multi-page results
    show_deleted: Option<bool>,
    show_hidden: Option<bool>,

    // don't need to touch this - but it has to be true for startTime ordering to work
    #[builder(skip = true)]
    single_events: bool, // will essentially expand recurring events into separate instances which is what I *always* want
}

// unfortunately, in the current design can't support Cow<'a, str> so building these Para isn't cheap :(
impl EventsListGetPara {
    pub fn change_page_token(&mut self, token: String) {
        self.page_token = Some(token);
    }
}

impl QueryParameters for EventsListGetPara {}

#[skip_serializing_none]
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
    // WARNING; timezone not needed for recurring events and if you don't specify timezone, then make sure to add timezone offset manually in the DateTime
    #[builder(into)]
    start: CalDateTime,
    #[builder(into)]
    end: CalDateTime,

    summary: Option<String>,     // if I want to change the title
    description: Option<String>, // for wind down notes
    color_id: String,            // make this required
}
impl QueryPayload for EventPayload {}

// endpoint!(GCalServer, pub Calendars, "",CalendarsPart, CalendarsRes,(), (), GET);
endpoint!(GCalServer, pub EventsListGet, "", EventsPart, GCalApiRes<ListRes<EventRes>>, EventsListGetPara, (), GET);
endpoint!(GCalServer, pub EventUpdate, "", EventsPartWithId, GCalApiRes<EventRes>, (), EventPayload, PUT);
endpoint!(GCalServer, pub EventDelete, "", EventsPartWithId, GCalApiRes<()>, (), (), DELETE);
endpoint!(GCalServer, pub EventAdd, "", EventsPart, GCalApiRes<EventRes>, (), EventPayload, POST); // insert does support parameters, but nothing important for this app

// NOTE; google says calendar id and event id are parameters - but these are inside the URL not here

// TODO; research watch endpoint as well
