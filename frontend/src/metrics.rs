/*//! metrics.rs
//!
//! not currently used

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use common_lib::common_structs::{QueryAverage, SESSION_USERNAME};
use common_lib::http::redirect_home;
use handlebars::Handlebars;
use serde_json::json;
use sqlx::PgPool;

/// authorization: required
pub async fn get_avg(
    hb: web::Data<Handlebars<'_>>,
    db_pool: web::Data<PgPool>,
    session: Session,
) -> impl Responder {
    if let Ok(Some(session_username)) = session.get::<String>(SESSION_USERNAME) {
        tracing::debug!("session id: {}", &session_username);
        let json = get_data(&db_pool).await;

        let data = json!({
            "title": "1-minute ticker",
            "parent": "base0",
            "is_logged_in": true,
            "session_username": &session_username,
            "data"  : json
        });
        let body = hb.render("avg", &data).unwrap();
        HttpResponse::Ok()
            .append_header(("cache-control", "no-store"))
            .body(body)
    } else {
        redirect_home().await
    }
}

/// authorization: required
pub async fn get_chart(
    hb: web::Data<Handlebars<'_>>,
    db_pool: web::Data<PgPool>,
    session: Session,
) -> impl Responder {
    tracing::debug!("[get_chart]");
    if let Ok(Some(session_username)) = session.get::<String>(SESSION_USERNAME) {
        tracing::debug!("session id: {}", &session_username);
        let json = get_data(&db_pool).await;
        // tracing::debug!("[get_chart] data: {:?}", &json);
        let data = json!({
            "title": "1-Minute Chart",
            "parent": "base0",
            "is_logged_in": true,
            "session_username": &session_username,
            "data"  : json
        });
        let body = hb.render("chart", &data).unwrap();
        HttpResponse::Ok()
            .append_header(("cache-control", "no-store"))
            .body(body)
    } else {
        redirect_home().await
    }
}

/// placeholder
///
/// json of Vec<QueryAverage>
async fn get_data(db_pool: &web::Data<PgPool>) -> String {
    // exclamation point means we're overriding sqlx requiring Option<> on nullables (assuming we know it'll never be null)
    let result_vector = sqlx::query_as!(
        QueryAverage,
        r#"
            select
                dtg as "dtg!",
                coalesce(symbol, '') as "symbol!",
                price_close as "price!"
                -- size as "size!",
                -- exchange as "exchange"
            from
                -- v_trade_minute
                bar_minute
            order by dtg desc
            limit 10000
        "#,
    )
    .fetch_all(db_pool.as_ref())
    .await
    .unwrap();

    json!(result_vector).to_string()
}
*/