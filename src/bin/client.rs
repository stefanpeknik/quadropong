use log::{error, info};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::{
    io::{self, Stderr},
    path::PathBuf,
};

use quadropong::client::{app::App, config::Config, error::ClientError};

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

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stderr>>, io::Error> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
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
        error!("Failed to setup logger (not critical, continuing): {}", e);
    }

    let config = if let Ok(config) = Config::load_config() {
        config
    } else {
        error!("Failed to load settings, using default one");
        Config::default()
    };

    let mut terminal = setup_terminal()?;

    let app_running = (|| async {
        let mut app = App::new(&mut terminal, config)?;
        app.run().await?;
        Ok::<(), ClientError>(())
    })()
    .await;

    restore_terminal(&mut terminal)?;

    app_running
}
