use datafusion::prelude::SessionContext;

#[derive(Clone)]
pub struct AppState {
    pub ctx: SessionContext,
}

impl AppState {
    pub fn new(ctx: SessionContext) -> Self {
        Self { ctx }
    }
}