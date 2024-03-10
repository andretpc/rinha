mod app_config;
mod app_error;
mod app_state;
mod balance;
mod client;
mod handlers;
mod statement;
mod transaction;
mod utils;

use app_config::config;
use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use hyperlocal::UnixServerExt;
use std::os::unix::fs::PermissionsExt;
use std::{fs::remove_file, path};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = config();

    let app_state = AppState::new(&config).await;

    let app = Router::new()
        .route("/clientes/:id/extrato", get(handlers::statement))
        .route("/clientes/:id/transacoes", post(handlers::transaction))
        .with_state(app_state);

    let path = path::Path::new(config.socket_path.as_str());

    if path.exists() {
        remove_file(path).expect("Could not remove old socket!");
    }

    let builder = axum::Server::bind_unix(path).unwrap();

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o777)).unwrap();

    builder.serve(app.into_make_service()).await.unwrap();
}
