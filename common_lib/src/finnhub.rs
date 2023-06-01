//! finnhub.rs


use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize,Serialize};
use chrono::serde::ts_milliseconds;

#[derive(PartialEq)]
pub enum FinnhubStream {
    TextData,
    BinaryUpdates,
}

#[derive(Serialize, Debug)]
pub struct FinnhubSubscribe{
    #[serde(rename="type")]
    pub websocket_message_type:String,
    pub symbol:String,
}

/// https://finnhub.io/docs/api/websocket-trades
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinnhubTrade{

    #[serde(rename="t")]
    #[serde(with = "ts_milliseconds")]
    pub dtg: DateTime<Utc>,

    #[serde(rename="s")]
    pub symbol: String,

    #[serde(rename="p")]
    pub price:BigDecimal,

    #[serde(rename="v")]
    pub volume:BigDecimal,

    #[serde(rename="c")]
    pub conditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinnhubData{
    pub data:Vec<FinnhubTrade>
}