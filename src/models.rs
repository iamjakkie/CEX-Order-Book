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