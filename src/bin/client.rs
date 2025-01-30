use std::io;

use quadropong::client::{app::App, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    if let Ok(settings) = Settings::load_config() {
        let mut app = App::new(settings);
        app.run().await?;
    }

    Ok(())
}
