use bon::Builder;
use restman_rs::{
    GET, endpoint,
    request::{QueryParameters, QueryParametersOptional},
};
use serde::{Deserialize, Serialize};

use crate::gcal::{
    GCalServer,
    api::{GCalApiRes, endpoints::ListRes, request::Me},
};

#[derive(Deserialize, Debug)]
pub struct CalendarRes {
    // again cherry picked the ones I *need*
    pub etag: String,
    pub id: String,      // need this for api
    pub summary: String, // this is the user-specified name
}

#[derive(Serialize, Builder)]
pub struct CalendarListPara {
    max_result: Option<usize>,
    page_token: Option<String>,

    show_deleted: Option<bool>,
    show_hidden: Option<bool>,
    // several others but these were the important ones (refer to https://developers.google.com/workspace/calendar/api/v3/reference/calendarList/list)
}
impl QueryParameters for CalendarListPara {}
impl QueryParametersOptional for CalendarListPara {} // since none of them are needed, the whole is optional!

endpoint!(GCalServer, pub CalendarListGet, "calendarList", Me, GCalApiRes<ListRes<CalendarRes>>, CalendarListPara, (), GET);
// my application doesn't need to DELETE, INSERT etc
