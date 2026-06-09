use std::{net::SocketAddr, str::FromStr};

use tokio::sync::mpsc;

use crate::{
    bark::builder::BarkClientBuilder, brain::pages::Homepage, gcal::GoogleCalendar,
    server::builder::OodServerBuilder,
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
const OOD_SERVER_URI: &str = "192.168.0.105:3002";

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

    let mut gcal = GoogleCalendar::builder(
        google_client_id,
        google_client_secret,
        SocketAddr::from_str(GOOGLE_REDIRECT_URI).unwrap(),
    )
    .login(google_my_refresh_token)
    .await
    .unwrap();

    let bark = BarkClientBuilder::new(bark_key).build();

    let (list_tx, mut list_rx) = mpsc::channel(1);

    let server = OodServerBuilder::new(SocketAddr::from_str(OOD_SERVER_URI).unwrap())
        .add_route(Homepage::new(list_tx))
        .start_server();

    let mut names = vec![];
    loop {
        let name = list_rx.recv().await.unwrap();
        names.push(name);
        println!("{names:?}")
    }
}
