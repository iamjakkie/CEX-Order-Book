mod models;

use models::{Instrument, OkxResponse, Spread};
use websocket::ClientBuilder;

fn fetch_instruments() -> Vec<Instrument> {
    let mut instruments = vec![];
    let client = reqwest::blocking::Client::new();
    let mut req = client.get("https://www.okx.com/api/v5/public/instruments");

    for inst_type in ["SPOT", "SWAP"].iter() {
        let req_with_query = req.try_clone().expect("Failed to clone request").query(&[("instType", inst_type)]);

        let resp = req_with_query.send().unwrap();
        if resp.status().is_success() {
            let parsed: OkxResponse<Instrument> = resp.json().unwrap();
            instruments.extend(parsed.data);
        } else {
            eprintln!("❌ Failed to fetch instruments: {}", resp.status());
        }
    }
    instruments
}

fn main() {
    let ws_url = "wss://ws.okx.com:8443/ws/v5/business";

    let instruments = fetch_instruments();

    println!("Spreads: {:?}", instruments);

    println!("Length: {}", instruments.len());


}
