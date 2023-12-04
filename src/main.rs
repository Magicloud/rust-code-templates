mod parameters;
mod handlers;
mod app_state;

use std::sync::Arc;

use axum::{ Router, routing::{ post, get } };
use clap::Parser;
use diesel_async::{
    pooled_connection::{ AsyncDieselConnectionManager, bb8::Pool },
    AsyncPgConnection,
};
use bb8_async_memcached::MemcacheConnectionManager;
use percent_encoding::{ utf8_percent_encode, NON_ALPHANUMERIC };
use crate::{ parameters::*, handlers::*, app_state::* };

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Params::parse();

    let mc_manager = MemcacheConnectionManager { uri: args.memcached_address };
    let mc_pool = bb8::Pool::builder().build(mc_manager).await?;

    let pg_conn_str = format!(
        "postgres://{}:{}@{}:{}/{}",
        args.pg_params.pg_username,
        utf8_percent_encode(&args.pg_params.pg_password, NON_ALPHANUMERIC),
        args.pg_params.pg_host,
        args.pg_params.pg_port,
        args.pg_params.pg_database
    );
    let pg_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(pg_conn_str);
    let pg_pool = Pool::builder().build(pg_manager).await?;

    let state = AppState {
        mc_pool,
        pg_pool,
    };

    let app = Router::new()
        .route("/heart_beat", get(heart_beat))
        .route("/status", get(status))
        .with_state(Arc::new(state));
    let listener = tokio::net::TcpListener::bind(args.listen_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
