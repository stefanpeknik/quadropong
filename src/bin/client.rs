use std::io;

use quadropong::client::app::App;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut app = App::new();
    app.run().await?;

    Ok(())
}
