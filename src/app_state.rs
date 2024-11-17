use aws_sdk_s3::Client;
use datafusion::prelude::SessionContext;

#[derive(Clone)]
pub struct AppState {
    pub ctx: SessionContext,
    pub client: Client,
}

impl AppState {
    pub fn new(ctx: SessionContext, client: Client) -> Self {
        Self { ctx, client }
    }
}