mod models;

use models::{OkxResponse, Spread};
use websocket::ClientBuilder;

fn fetch_spreads() -> Vec<Spread> {
    let client = reqwest::blocking::Client::new();
    let mut req = client.get("https://www.okx.com/api/v5/sprd/spreads");


    let resp = req.send().unwrap();

    if resp.status().is_success() {
        let parsed: OkxResponse<Spread> = resp.json().unwrap();
        parsed.data
    } else {
        eprintln!("❌ Failed to fetch spreads: {}", resp.status());
        vec![]
    }
}

fn main() {
    let ws_url = "wss://ws.okx.com:8443/ws/v5/business";

    let spreads = fetch_spreads();

    println!("Spreads: {:?}", spreads);


}
