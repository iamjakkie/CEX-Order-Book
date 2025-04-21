use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OkxResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Spread {
    pub sprdId: String,
    pub sprdType: String,
    pub state: String,
    pub baseCcy: String,
    pub szCcy: String,
    pub quoteCcy: String,
    pub tickSz: String,
    pub minSz: String,
    pub lotSz: String,
    pub listTime: String,
    pub expTime: String,
    pub uTime: String,
    pub legs: Vec<Leg>,
}

#[derive(Debug, Deserialize)]
pub struct Leg {
    pub instId: String,
    pub side: String,
}

#[derive(Debug, Deserialize)]
pub struct Instrument {
    pub instType: Option<String>,
    pub instId: Option<String>,
    pub uly: Option<String>,
    pub instFamily: Option<String>,
    pub category: Option<String>,
    pub baseCcy: Option<String>,
    pub quoteCcy: Option<String>,
    pub settleCcy: Option<String>,
    pub ctVal: Option<String>,
    pub ctMult: Option<String>,
    pub ctValCcy: Option<String>,
    pub optType: Option<String>,
    pub stk: Option<String>,
    pub listTime: Option<String>,
    pub auctionEndTime: Option<String>,
    pub expTime: Option<String>,
    pub lever: Option<String>,
    pub tickSz: Option<String>,
    pub lotSz: Option<String>,
    pub minSz: Option<String>,
    pub ctType: Option<String>,
    pub alias: Option<String>,
    pub state: Option<String>,
    pub ruleType: Option<String>,
    pub maxLmtSz: Option<String>,
    pub maxMktSz: Option<String>,
    pub maxLmtAmt: Option<String>,
    pub maxMktAmt: Option<String>,
    pub maxTwapSz: Option<String>,
    pub maxIcebergSz: Option<String>,
    pub maxTriggerSz: Option<String>,
    pub maxStopSz: Option<String>,
    pub futureSettlement: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BookMsg {
    pub arg: BookArg,
    pub action: Option<String>, // snapshot or update
    pub data: Vec<BookData>,
    pub ts: i64,
    pub checksum: Option<i64>,
    pub seqId: Option<i64>,
    pub prevSeqId: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct WsBookPush {
    pub arg: BookArg,
    pub action: Option<String>,
    pub data: Vec<BookData>,
}

#[derive(Debug, Deserialize)]
pub struct BookArg {
    pub channel: String,
    pub instId: String,
}

#[derive(Debug, Deserialize)]
pub struct BookData {
    pub asks: Vec<[String; 4]>,
    pub bids: Vec<[String; 4]>,
    pub ts: String,
    pub checksum: Option<i64>,
    pub seqId: Option<i64>,
    pub prevSeqId: Option<i64>,
}