use chrono::{DateTime, Utc};
use restman_rs::{
    ADelete, AGet, APost, ApiBackendError, ApiHttpClient, AsyncMethodMarkerGetter,
    client::async_client::ApiClient,
    request::{ApiPayload, ApiRequest, QueryPayload, ValidRequest, endpoints::Endpoint},
};

use crate::gcal::{
    GCalErr, GCalServer, GoogleCalendar,
    api::{
        GCalApiRes, GCalConfig,
        endpoints::{
            ListRes,
            calendar_list::{CalendarListGet, CalendarListPara, CalendarRes},
            colors::{ColorsGet, ColorsRes},
            events::{
                EventAdd, EventDelete, EventPayload, EventRes, EventsListGet, EventsListGetPara
            },
        },
    },
};

const HTTP_UNAUTHORISED: u16 = 401;

impl<C: ApiHttpClient + Sync> GoogleCalendar<C> {
    // it has to be mutable because we may have to refresh token
    async fn request<E, P, R>(&mut self, r: &R) -> Result<P, GCalErr<C>>
    where
        E: Endpoint<Ser = GCalServer, Payload = (), Res = GCalApiRes<P>>,
        E::Method: AsyncMethodMarkerGetter<C>,
        R: ValidRequest<E>,
        R: Sync,
    {
        match self.backend().async_request(r).await?.into_result() {
            Ok(res) => Ok(res),
            Err(e) => {
                if e.code == HTTP_UNAUTHORISED {
                    // code may be expired
                    println!("Code invalid - trying to refresh");
                    self.refresh_token().await?;
                    Ok(self.backend().async_request(r).await?.into_result()?)
                } else {
                    Err(e)?
                }
            }
        }
    }

    async fn send_payload<E, P, R>(
        &mut self,
        r: &R,
        p: &ApiPayload<E::Payload>,
    ) -> Result<P, GCalErr<C>>
    where
        E: Endpoint<Ser = GCalServer, Res = GCalApiRes<P>>,
        E::Method: AsyncMethodMarkerGetter<C>,
        E::Payload: QueryPayload,
        R: ValidRequest<E>,
        R: Sync,
    {
        match self.backend().async_send_payload(r, p).await?.into_result() {
            Ok(res) => Ok(res),
            Err(e) => {
                if e.code == HTTP_UNAUTHORISED {
                    // code may be expired
                    println!("Code invalid - trying to refresh");
                    self.refresh_token().await?;
                    Ok(self
                        .backend()
                        .async_send_payload(r, p)
                        .await?
                        .into_result()?)
                } else {
                    Err(e)?
                }
            }
        }
    }
}

impl<C: ApiHttpClient + AGet + Sync> GoogleCalendar<C> {
    pub async fn find_calendar(&mut self, name: &str) -> Result<CalendarRes, GCalErr<C>> {
        const CALENDAR_N: usize = 5; // first five will almost certainly be right

        let mut req = ApiRequest::<CalendarListGet>::new(&())
            .add_para(&CalendarListPara::builder().max_result(CALENDAR_N).build());
        let mut res = self.request(&req).await?;
        loop {
            let ListRes {
                next_page_token,
                items,
                ..
            } = res;
            if let Some(entry) = items.into_iter().find(|e| e.summary == name) {
                return Ok(entry);
            } else if let Some(next_page) = next_page_token {
                req.change_para(&CalendarListPara::builder().page_token(next_page).build());
                res = self.request(&req).await?;
            } else {
                return Err(GCalErr::CalendarNotFound);
            }
        }
    }

    pub async fn get_events(
        &mut self,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
    ) -> Result<Vec<EventRes>, GCalErr<C>> {
        let mut para = EventsListGetPara::builder()
            .time_min(time_min)
            .time_max(time_max)
            .build();

        let mut events = vec![];

        let mut req = ApiRequest::<EventsListGet>::new(&GCalConfig::new().calendar_id(calendar_id))
            .add_para(&para);

        loop {
            let ListRes {
                next_page_token,
                items,
                ..
            } = self.request(&req).await?;
            events.extend(items);

            let Some(next_page_token) = next_page_token else {
                break;
            };

            para.change_page_token(next_page_token); // HACK: because I can't figure out how to cleanly support Cow<'a, str> yet
            req.change_para(&para);
        }
        Ok(events)
    }

    pub async fn get_colors(&mut self) -> Result<ColorsRes, GCalErr<C>> {
        Ok(self.request(&ApiRequest::<ColorsGet>::new(&())).await?)
    }

    pub async fn insert_event(
        &mut self,
        calendar_id: &str,
        payload: EventPayload,
    ) -> Result<EventRes, GCalErr<C>>
    where
        C: APost,
    {
        let req = ApiRequest::<EventAdd>::new(&GCalConfig::new().calendar_id(calendar_id));
        let payload = ApiPayload::new(&payload)?;
        Ok(self.send_payload(&req, &payload).await?)
    }

    pub async fn delete_event(
        &mut self,
        event_id: &str,
        calendar_id: &str,
    ) -> Result<(), GCalErr<C>>
    where
        C: ADelete,
    {
        let res = self
            .request(&ApiRequest::<EventDelete>::new(
                &GCalConfig::new()
                    .event_id(event_id)
                    .calendar_id(calendar_id),
            ))
            .await;
        match res {
            // this is OK - google returns empty response if success for this request
            Err(GCalErr::ApiBackendErr(ApiBackendError::ParseError(e))) if e.is_eof() => Ok(()),
            e => Ok(e?),
        }
    }
}
