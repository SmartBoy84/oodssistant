use super::{HasCalendarID, HasEventID};
use restman_rs::request_part;

request_part!(Calendar, "calendar", ());
request_part!(V3, "v3", Calendar);
request_part!(Users, "users", V3);
request_part!(Me, "me", Users); // is this configurable? not sure, docs don't say

// events needs event id
request_part!(CalendarsPart, "calendars", V3, HasCalendarID, calendar_id);
request_part!(EventsPart, "events", CalendarsPart);
request_part!(EventsPartWithId, "", EventsPart, HasEventID, event_id);
