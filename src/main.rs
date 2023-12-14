#![feature(try_blocks)]

mod parameters;
mod handlers;
mod app_state;

use std::sync::Arc;
use anyhow::Result;
use axum::{ Router, routing::get };
use clap::Parser;
use diesel_async::{
    pooled_connection::{ AsyncDieselConnectionManager, bb8::Pool },
    AsyncPgConnection,
    AsyncConnection,
};
use tower_http::trace::TraceLayer;
use tracing::{ info, instrument, error };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt };
use crate::{ parameters::*, handlers::*, app_state::* };
use bb8_async_memcached::MemcacheConnectionManager;
use percent_encoding::{ utf8_percent_encode, NON_ALPHANUMERIC };

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber
        ::registry()
        .with(
            tracing_subscriber::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| "storing=info,tower_http=trace".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Params::parse();

    let mc_result: Result<_> = try {
        let mc_manager = MemcacheConnectionManager { uri: args.memcached_address };
        let mc_pool = bb8::Pool::builder().build(mc_manager).await?;
        mc_pool.get().await?.version().await?;
        mc_pool
    };
    if let Err(e) = mc_result {
        error!("memcache error: {e:?}");
        return Err(e);
    }
    let mc_pool = mc_result.expect("msg");

    let pg_result: Result<_> = try {
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
        pg_pool.get().await?.begin_test_transaction().await?;
        pg_pool
    };
    if let Err(e) = pg_result {
        error!("postgresql error: {e:?}");
        return Err(e);
    }
    let pg_pool = pg_result.expect("msg");

    let state = Arc::new(AppState {
        mc_pool,
        pg_pool,
    });

    let app = Router::new()
        .route("/heart_beat", get(heart_beat))
        .route("/status", get(status))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(args.listen_address).await.unwrap();

    info!("Starting Axum");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
