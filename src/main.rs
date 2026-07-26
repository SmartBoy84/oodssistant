use std::{net::SocketAddr, str::FromStr};

use crate::{
    bark::builder::BarkClientBuilder, brain::{Ood, pages::Homepage}, gcal::GoogleCalendar,     server::{
        OodServer,
        builder::OodServerBuilder,
        interface::page::{basic::OodStatic, para::OodPara},
    },
};

mod bark;
mod brain;
mod gcal;
mod server;

const BARK_KEY: &str = env!("BARK_KEY");
const GOOGLE_CLIENT_ID: &str = env!("GOOGLE_CLIENT_ID");
const GOOGLE_CLIENT_SECRET: &str = env!("GOOGLE_CLIENT_SECRET");
const GOOGLE_MY_REFRESH_TOKEN: &str = env!("GOOGLE_MY_REFRESH_TOKEN");

const GOOGLE_REDIRECT_URI: &str = "127.0.0.1:3001";
const OOD_SERVER_URI: &str = "192.168.0.105:3002";

const OOD_CALENDAR_NAME: &str = "Ood";

#[tokio::main]
async fn main() {
    // let gcal = GoogleCalendar::builder(
    //     GOOGLE_CLIENT_ID,
    //     GOOGLE_CLIENT_SECRET,
    //     SocketAddr::from_str(GOOGLE_REDIRECT_URI).unwrap(),
    // )
    // .login(GOOGLE_MY_REFRESH_TOKEN)
    // .await
    // .unwrap();

    // let bark = BarkClientBuilder::new(BARK_KEY).build();

    let server = OodServerBuilder::new(SocketAddr::from_str(OOD_SERVER_URI).unwrap());
    server.add_route(OodStatic(Homepage::default())).start_server().await_server().await;

    // Ood::new(gcal, bark, server, OOD_CALENDAR_NAME)
    //     .await
    //     .unwrap()
    //     .run_me()
    //     .await
    //     .unwrap();
}
