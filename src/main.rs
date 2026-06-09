use std::{net::SocketAddr, str::FromStr};

use tokio::sync::mpsc;

use crate::{
    bark::builder::BarkClientBuilder,
    brain::test::homepage::{DynamicStaticTest, Homepage, ParaPageTest},
    gcal::GoogleCalendar,
    server::{
        builder::OodServerBuilder,
        interface::page::{basic::OodStatic, para::OodPara},
    },
};

mod bark;
mod brain;
mod gcal;
mod server;

const SECRETS_FILE: &str = "SECRETS.env";

// Bark data
const BARK_KEY: &str = "BARK_KEY";

// all of the following are app specific (including redirect uri!)
const GOOGLE_CLIENT_ID: &str = "GOOGLE_CLIENT_ID";
const GOOGLE_CLIENT_SECRET: &str = "GOOGLE_CLIENT_SECRET";

// all of the following is client data
const GOOGLE_MY_REFRESH_TOKEN: &str = "GOOGLE_MY_REFRESH_TOKEN";

const GOOGLE_REDIRECT_URI: &str = "127.0.0.1:3001";
const OOD_SERVER_URI: &str = "127.0.0.1:3002";

const OOD_SHORTCUT_NAME: &str = "ood";

#[tokio::main]
async fn main() {
    let data = std::fs::read_to_string(SECRETS_FILE).unwrap();

    let mut bark_key = "";
    let mut google_client_id = "";
    let mut google_client_secret = "";
    let mut google_my_refresh_token = "";
    for (name, key) in data.lines().filter_map(|l| l.split_once('=')) {
        match name {
            BARK_KEY => bark_key = key,
            GOOGLE_CLIENT_ID => google_client_id = key,
            GOOGLE_CLIENT_SECRET => google_client_secret = key,
            GOOGLE_MY_REFRESH_TOKEN => google_my_refresh_token = key,
            _ => println!("uh, unknown pair: {name}={key}"),
        }
    }

    // let mut gcal = GoogleCalendar::builder(
    //     google_client_id,
    //     google_client_secret,
    //     SocketAddr::from_str(GOOGLE_REDIRECT_URI).unwrap(),
    // )
    // .login(google_my_refresh_token)
    // .await
    // .unwrap();

    // let bark = BarkClientBuilder::new(bark_key).build();

    let server = OodServerBuilder::new(SocketAddr::from_str(OOD_SERVER_URI).unwrap())
        .add_route(OodStatic(Homepage::new(OOD_SHORTCUT_NAME)))
        .add_route(OodStatic(DynamicStaticTest("/a")))
        .add_route(OodStatic(DynamicStaticTest("/b")))
        .add_route(OodPara(ParaPageTest))
        .start_server()
        .await_server()
        .await
        .unwrap();
}
