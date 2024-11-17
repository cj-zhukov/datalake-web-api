use anyhow::Result;

use datalake_web_api::Application;
use datalake_web_api::utils::constants::test;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Application::build(test::APP_ADDRESS).await?;
    app.run().await?;

    Ok(())
}