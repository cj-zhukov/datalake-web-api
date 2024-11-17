use std::{env as std_env, sync::LazyLock};

use dotenvy::dotenv;

pub mod env {
    pub const BUCKET_SELECT_ENV_VAR: &str = "BUCKET_SELECT_SECRET";
    pub const PREFIX_SELECT_ENV_VAR: &str = "PREFIX_SELECT_SECRET";
    pub const BUCKET_DOWNLOAD_ENV_VAR: &str = "BUCKET_DOWNLOAD_SECRET";
}

pub const REGION: &str = "eu-central-1";
pub const TABLE_NAME: &str = "t";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:80";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:8080";
}

pub static BUCKET_SELECT: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::BUCKET_SELECT_ENV_VAR)
        .expect("BUCKET_SELECT must be set.");
    if secret.is_empty() {
        panic!("BUCKET_SELECT must not be empty.");
    }
    secret
});

pub static PREFIX_SELECT: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::PREFIX_SELECT_ENV_VAR)
        .expect("PREFIX_SELECT_ENV_VAR must be set.");
    if secret.is_empty() {
        panic!("PREFIX_SELECT_ENV_VAR must not be empty.");
    }
    secret
});

pub static BUCKET_DOWNLOAD: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::BUCKET_DOWNLOAD_ENV_VAR)
        .expect("BUCKET_DOWNLOAD_ENV_VAR must be set.");
    if secret.is_empty() {
        panic!("BUCKET_DOWNLOAD_ENV_VAR must not be empty.");
    }
    secret
});