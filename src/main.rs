use irondom::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = irondom::config::Config::load().unwrap_or_default();
    let port = config.http_port;
    let state = app::build_state(config).await?;
    irondom::automation::scheduler::spawn(state.automation_engine.clone());
    let app = app::router(state);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    tracing::info!("irondom listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
