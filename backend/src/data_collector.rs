//! backend

/*

    Market

    Starts both REST and websocket services.

    Starts a DB thread to store results of rest/ws tickers.

    Performs analysis on incoming tickers.


*/

use crate::db::DbActor;
use crate::websocket_service::AlpacaStream;
use crate::ws_finnhub::WsFinnhub;
use common_lib::finnhub::FinnhubStream;
use common_lib::settings::Settings;
use common_lib::symbol_list::SymbolList;
use sqlx::PgPool;
use std::str::FromStr;

pub struct DataCollector {}

impl DataCollector {
    pub async fn start(pool: PgPool, settings: &Settings) {
        // old: phase this out for separate microservice and pull data directly from message broker
        // Postgres Database
        // Start the long-running database thread;
        // get a sender from the Database Service.
        // tx = "transmitter"
        let tx_db = DbActor::start().await;
        tracing::debug!("[Market::start] db start() complete");
        // Websocket (Incoming) Data Service
        let alpaca_ws_on = bool::from_str(
            std::env::var("ALPACA_WEBSOCKET_ON")
                .unwrap_or_else(|_| "true".to_owned())
                .as_str(),
        )
        .unwrap_or(false);
        tracing::info!("ALPACA_WEBSOCKET_ON is: {}", &alpaca_ws_on);

        if alpaca_ws_on {
            // spawn long-running text thread
            let tx_db_ws = tx_db.clone();
            tracing::debug!("Starting text websocket service in new thread...");
            let ws_pool = pool.clone();

            let settings2 = (*settings).clone();
            match SymbolList::get_active_symbols(&ws_pool).await {
                Ok(symbols) => {
                    let _ = std::thread::spawn(move || {
                        crate::websocket_service::Ws::run(
                            tx_db_ws,
                            &AlpacaStream::TextData,
                            symbols,
                            settings2,
                        );
                    });
                }
                Err(e) => {
                    tracing::debug!("[start] error getting symbols for websocket: {:?}", &e);
                }
            }
        } else {
            tracing::debug!("Websocket service not started, ALPACA_WEBSOCKET_ON is false");
        }

        let finnhub_on = bool::from_str(
            std::env::var("FINNHUB_ON")
                .unwrap_or_else(|_| "true".to_owned())
                .as_str(),
        )
        .unwrap_or(true);
        tracing::info!("FINNHUB_ON is: {}", &finnhub_on);

        if finnhub_on {
            // spawn long-running text thread
            let tx_db_ws = tx_db.clone();
            tracing::debug!("Starting Finnhub text websocket service in new thread...");
            let ws_pool = pool.clone();

            let settings2 = (*settings).clone();
            match SymbolList::get_active_symbols(&ws_pool).await {
                Ok(symbols) => {
                    tokio::spawn(async move {
                        WsFinnhub::run(tx_db_ws, &FinnhubStream::TextData, symbols, settings2)
                            .await;
                    });
                }
                Err(e) => {
                    tracing::debug!("[start] error getting symbols for websocket: {:?}", &e);
                }
            }
        } else {
            tracing::debug!("Websocket service not started, ALPACA_WEBSOCKET_ON is false");
        }

        // Rest HTTP Service (in/out)
        let alpaca_rest_on = bool::from_str(
            std::env::var("ALPACA_REST_ON")
                .unwrap_or_else(|_| "false".to_owned())
                .as_str(),
        )
        .unwrap_or(false);
        tracing::info!("ALPACA_REST_ON is: {}", alpaca_rest_on);

        if alpaca_rest_on {
            tracing::debug!("[Market::start] starting alpaca web client");
            crate::rest_client::run(/*stocks, tx_db, tx_trader*/).await;
            tracing::debug!("[Market::start] alpaca web client finished");
        } else {
            tracing::debug!("Rest service not started, ALPACA_REST_ON is false");
        }

        // infinite loop to keep child threads alive
        loop {
            // TODO: separate rest and websocket threads so we don't have to deal with this
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
