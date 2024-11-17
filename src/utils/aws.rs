use std::time::Duration;

use anyhow::Result;
use aws_config::{retry::RetryConfig, timeout::TimeoutConfig, BehaviorVersion, Region};
use aws_sdk_s3::{operation::get_object::GetObjectOutput, Client};
use aws_sdk_s3::config::Builder;

pub async fn get_aws_client(region: &str) -> Client {
    let region = Region::new(region.to_string());

    let sdk_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    let timeout= TimeoutConfig::builder()
        .operation_timeout(Duration::from_secs(60 * 5))
        .operation_attempt_timeout(Duration::from_secs(60 * 5))
        .connect_timeout(Duration::from_secs(60 * 5))
        .build();

    let config_builder = Builder::from(&sdk_config)
        .timeout_config(timeout)
        .retry_config(RetryConfig::standard().with_max_attempts(10));

    let config = config_builder.build();
   
    Client::from_conf(config)
}


pub async fn get_aws_object(client: Client, bucket: &str, key: &str) -> Result<GetObjectOutput> {
    let req = client
        .get_object()
        .bucket(bucket)
        .key(key);

    let res = req.send().await?;

    Ok(res)
}

pub async fn read_file(client: Client, bucket: String, key: String) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut object = get_aws_object(client, &bucket, &key).await?;
    while let Some(bytes) = object.body.try_next().await? {
        buf.extend(bytes.to_vec());
    }

    Ok(buf)
}