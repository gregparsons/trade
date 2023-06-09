//! init.rs

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init(package_name: &str) {
    // load .env
    // if docker, load .env from the root directory, otherwise use the cargo workspace directory
    let config_location =
        std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned());
    println!("[init] config_location: {}", &config_location);
    let dot_env_path = match config_location.as_str() {
        "docker" => ".env".to_string(),
        "not_docker" | _ => {
            // backend/.env
            // frontend/.env
            // etc
            format!("{}/.env", &package_name)
        }
    };
    match dotenvy::from_filename(&dot_env_path) {
        Ok(_) => println!("[init] .env found"),
        _ => println!("[init] .env not found"),
    }
    let env_file_version =
        std::env::var("ENV_FILE_VERSION").unwrap_or_else(|_| ".env not loaded".to_string());
    println!(
        "[init] dot_env_path: {}; env_file_version: {}",
        &dot_env_path, &env_file_version
    );

    // start tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::info!("[init] .env file: {}", &dot_env_path);
}
