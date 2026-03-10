mod app;
mod config;
mod domain;
mod error;
mod http;
mod infra;
mod services;
mod state;

#[tokio::main]
async fn main() {
    if let Err(err) = app::run().await {
        eprintln!("fatal error: {err}");
        std::process::exit(1);
    }
}
