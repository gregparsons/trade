//! ws_finnhub.rs



use std::time::{Duration};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
/**
    Websocket client for Alpaca

    Current hard-coded stocks:
    aapl, tsla, plug, aal, nio, bac
*/
// use crossbeam_channel::{after, select, tick};
use crate::db::DbMsg;
use crossbeam::channel::Sender;
use serde_json::{json, Value};
use tungstenite::client::IntoClientRequest;
use tungstenite::{Message};
use common_lib::settings::Settings;
use common_lib::symbol_list::SymbolList;
use chrono::serde::ts_milliseconds;
use sqlx::PgPool;


#[derive(PartialEq)]
pub enum FinnhubStream {
    TextData,
    BinaryUpdates,
}

#[derive(Serialize, Debug)]
struct FinnhubSubscribe{
    #[serde(rename="type")]
    websocket_message_type:String,
    symbol:String,
}

/// https://finnhub.io/docs/api/websocket-trades
#[derive(Debug, Serialize, Deserialize)]
struct FinnhubTrade{

    #[serde(rename="t")]
    #[serde(with = "ts_milliseconds")]
    dtg: DateTime<Utc>,

    #[serde(rename="s")]
    symbol: String,

    #[serde(rename="p")]
    price:BigDecimal,

    #[serde(rename="v")]
    volume:BigDecimal,

    #[serde(rename="c")]
    conditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FinnhubData{
    data:Vec<FinnhubTrade>
}


fn stock_list_to_uppercase(lower_stock:&Vec<String>)-> Vec<String>{
    lower_stock.iter().map(|x| x.to_uppercase() ).collect()
}

pub struct WsFinnhub;

impl WsFinnhub {
    pub async fn run(tx_db: Sender<DbMsg>, stream_type:&FinnhubStream, symbols:Vec<String>, settings:Settings, pool:&PgPool) {
        tracing::debug!("[WsFinnhub::run]");
        WsFinnhub::connect(tx_db, stream_type, symbols, &settings, pool).await;
    }

    async fn connect(tx_db: Sender<DbMsg>, stream_type:&FinnhubStream, symbols:Vec<String>, settings:&Settings, pool:&PgPool) {

        // wss://ws.finnhub.io?token=xxxxxxxx
        // .env includes everything except the api key value (xxxxxx); called token here
        // TODO: binary not needed for finnhub
        let ws_url = match stream_type{
            FinnhubStream::TextData => std::env::var("FINNHUB_URL").expect("FINNHUB_URL not found"),
            FinnhubStream::BinaryUpdates => std::env::var("FINNHUB_URL").expect("FINNHUB_URL not found"),
        };

        let ws_url = format!("{}{}", ws_url, settings.finnhub_key);

        // websocket restart loop
        loop {

            let url = url::Url::parse(&ws_url).unwrap();
            let request = (&url).into_client_request().unwrap();

            // commence websocket connection
            match tungstenite::connect(request) {
                Err(e) => tracing::debug!("[WsFinnhub::connect] websocket connect error: {:?}", e),

                Ok((mut ws, _response)) => {
                    tracing::debug!("[WsFinnhub::connect] successful websocket connection; response: {:?}", _response);

                    // todo: check if websocket connected; it won't if there's one already connected elsewhere; Alpaca sends an error
                    // let auth_json = generate_ws_authentication_message(&settings);

                    // send authentication message
                    // ws.write_message(Message::Text(auth_json)).unwrap();

                    /*
                    2023-05-31T21:23:42.256121Z DEBUG backend::ws_finnhub: [WsFinnhub::connect] successful websocket connection; response: Response { status: 101, version: HTTP/1.1, headers: {"date": "Wed, 31 May 2023 21:23:43 GMT", "connection": "upgrade", "upgrade": "websocket", "sec-websocket-accept": "XLvDaH0hCELNbMnjEJFm/AZcf8I=", "cf-cache-status": "DYNAMIC", "report-to": "{\"endpoints\":[{\"url\":\"https:\/\/a.nel.cloudflare.com\/report\/v3?s=0B5jxuyY0Bc%2FaXpEeJ67xAOdM%2B4GMmAXGJpSdZuGlpB%2FzOVJLibsbfUL3Mf%2F1yZkFUAs%2BKX3KXRzpYmdq%2B%2FgXoRE81lt4TaesP1aUtcsP0eyDfrjMEL9yImHrXWfQzeU\"}],\"group\":\"cf-nel\",\"max_age\":604800}", "nel": "{\"success_fraction\":0,\"report_to\":\"cf-nel\",\"max_age\":604800}", "server": "cloudflare", "cf-ray": "7d0247931bbece94-SJC", "alt-svc": "h3=\":443\"; ma=86400"}, body: None }
                     */

                    // Subscribe to all symbols
                    for symbol in stock_list_to_uppercase(&symbols){
                        // {"type":"subscribe","symbol":"TSLA"}
                        let subscribe = json!(FinnhubSubscribe{
                            websocket_message_type: "subscribe".to_string(),
                            symbol
                        });
                        tracing::debug!("[WsFinnhub] subscribe: {}", &subscribe.to_string());
                        let _ = ws.write_message(Message::Text(subscribe.to_string()));
                    }

                    loop {
                        // tracing::debug!("[ws_connect] reading websocket...");
                        if let Ok(msg) = ws.read_message() {
                            tracing::debug!("[WsFinnhub::connect] read websocket...");
                            match msg {
                                Message::Ping(t) => {
                                    // not used by finnhub; they use json over Text
                                    tracing::debug!("[WsFinnhub::connect][ping] {:?}", &t);
                                },
                                Message::Binary(b_msg) => {
                                    tracing::debug!("[WsFinnhub::connect][binary] {:?}", &b_msg);
                                    // match serde_json::from_slice::<Value>(&b_msg) {
                                    //     Ok(json) => {
                                    //         tracing::debug!("[ws_connect][binary] binary json: {:?}", &json);
                                    //         tracing::debug!("[ws_connect][binary] json[data][stream].as_str(): {:?}", json["stream"].as_str());
                                    //         if json["stream"].as_str().unwrap() == "authorization" {
                                    //             if json["data"]["action"].as_str().unwrap() == "authenticate" &&
                                    //                 json["data"]["status"].as_str().unwrap() == "authorized" {
                                    //                 tracing::debug!("[ws_connect][binary] authorized");
                                    //
                                    //                 // SEND trade_updates request
                                    //                 let listen_msg = generate_ws_listen_message(vec!("trade_updates".to_string()));
                                    //                 tracing::debug!("[ws_connect][binary] outgoing listen msg: {}", &listen_msg);
                                    //                 let _ = ws.write_message(Message::Text(listen_msg));
                                    //             } else {
                                    //                 tracing::debug!("[ws_connect][binary] not authorized");
                                    //             }
                                    //         } else if json["stream"].as_str().unwrap() == "listening" {
                                    //             if let Ok(streams) = serde_json::from_value::<Vec<String>>(json["data"]["streams"].clone()) {
                                    //                 for stream in streams {
                                    //                     tracing::debug!("[ws_connect][binary] listening to stream: {}", &stream);
                                    //                 }
                                    //             }
                                    //         }
                                    //     },
                                    //     Err(e) => {
                                    //         tracing::debug!("[ws_connect][binary] binary json conversion error: {:?}", &e);
                                    //     }
                                    // }
                                }
                                Message::Text(t_msg) => {
                                    tracing::debug!("[WsFinnhub::connect][text] {}",&t_msg);


                                    // try turning this json into a FinnhubTrade (could also be a type/ping, so this would fail)
                                    match serde_json::from_str::<FinnhubData>(&t_msg){
                                        Ok(trade_data) => {

                                            for d in trade_data.data{

                                                tracing::debug!("[deserialize]: {:?}", &d);

                                                // TODO: not sure if sqlx/tokio async will block the websocket
                                                let _ = sqlx::query!(
                                                    r#"
                                                    insert into trade_fh (dtg,symbol, price, volume) values ($1, $2, $3, $4)
                                                    "#,
                                                    d.dtg,
                                                    d.symbol,
                                                    d.price,
                                                    d.volume
                                                ).execute(pool).await;
                                            }
                                        },
                                        Err(e)=>{
                                            tracing::debug!("[deserialize] error: {:?}", &e);
                                        }
                                    }



                                            // {"type":"ping"}
                                            // {"data":[{"c":["1","24","12"],"p":203.04,"s":"TSLA","t":1685567484229,"v":1},{"c":["1","24","12"],"p":203.05,"s":"TSLA","t":1685567484403,"v":1}],"type":"trade"}
                                            // {"data":[{"c":["1","24"],"p":203.04,"s":"TSLA","t":1685567486393,"v":500},{"c":["1","24","12"],"p":203.06,"s":"TSLA","t":1685567486611,"v":3},{"c":["1","24","12"],"p":177.46,"s":"AAPL","t":1685567486934,"v":72}],"type":"trade"}


                                    // let json_vec: Vec<Value> = serde_json::from_str(&t_msg).unwrap();
                                    // for json in json_vec {
                                    //
                                    //     if let Some(data) = json["data"].as_str() {
                                    //
                                    //         // parse pretend trade data
                                    //         // let pretend_data = r#"{"data":[{"c":["1","24"],"p":203.04,"s":"TSLA","t":1685567486393,"v":500},{"c":["1","24","12"],"p":203.06,"s":"TSLA","t":1685567486611,"v":3},{"c":["1","24","12"],"p":177.46,"s":"AAPL","t":1685567486934,"v":72}],"type":"trade"}"#;
                                    //         // let test = serde_json::from_str::<FinnhubData>(&pretend_data);
                                    //     }
                                    // }

                                }
                                _ => {
                                    tracing::debug!("[WsFinnhub::connect] websocket non-text, non-binary data: {:?}", &msg);
                                }
                            }
                        }
                    }
                }
            };

            // 5 second delay if the websocket goes down, then retry
            std::thread::sleep(Duration::from_millis(5000));

        }
    }



}

