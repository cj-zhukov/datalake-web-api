use color_eyre::Result;

use datalake_web_api::app_state::AppState;
use datalake_web_api::data_store::aws::init;
use datalake_web_api::Application;
use datalake_web_api::utils::constants::{test, REGION, BUCKET_SELECT, PREFIX_SELECT, TABLE_NAME};

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = init(REGION, BUCKET_SELECT.as_str(), PREFIX_SELECT.as_str(), TABLE_NAME).await?;
    let app_state = AppState::new(ctx);

    let app = Application::build(app_state, test::APP_ADDRESS).await?;
    app.run().await?;

    Ok(())
}