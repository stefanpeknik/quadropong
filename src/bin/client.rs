use std::io;

use quadropong::client::{app::App, config::Config};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    if let Ok(settings) = Config::load_config() {
        let mut app = App::new(settings);
        app.run().await?;
    }

    Ok(())
}
