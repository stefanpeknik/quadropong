use log::error;
use std::io;

use quadropong::client::{app::App, config::Config};

fn setup_logger() -> Result<(), fern::InitError> {
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
        .chain(fern::log_file("output.log")?) // Log to file
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    _ = setup_logger(); // Ignore logger failure
    if let Ok(settings) = Config::load_config() {
        let mut app = App::new(settings);
        app.run().await?;
    } else {
        error!("Failed to load settings");
    }

    Ok(())
}
