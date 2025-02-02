use log::{error, info};
use std::{io, path::PathBuf};

use quadropong::client::{app::App, config::Config};

fn setup_logger(log_path: PathBuf) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug) // Set global log level
        .chain(fern::log_file(log_path)?) // Log to file
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let log_dir = if let Some(path) = Config::get_log_path() {
        info!("Using log path: {}", path.display());
        path
    } else {
        error!("Failed to get config path, using current directory");
        std::env::current_dir().unwrap_or_else(|_| {
            error!("Failed to get current directory, using '.'");
            std::path::PathBuf::from(".")
        })
    };
    if let Err(e) = setup_logger(log_dir) {
        error!("Failed to setup logger: {}", e);
    }

    if let Ok(config) = Config::load_config() {
        let mut app = App::new(config);
        app.run().await?;
    } else {
        error!("Failed to load settings");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to load settings",
        ));
    }

    Ok(())
}
