use aws_sdk_s3::Client;
// use color_eyre::eyre::Result;
use awscreds::Credentials;
use datafusion::prelude::*;
// use datafusion::arrow::datatypes::{Int32Type};
use datafusion::arrow::array::AsArray;
use itertools::izip;
use object_store::aws::AmazonS3Builder;
use serde::{Serialize, Deserialize};
use tokio_stream::StreamExt;
use std::sync::Arc;
use url::Url;

use crate::utils::aws::read_file;
use crate::utils::datafusion::is_empty;
use super::error::DataStoreError;
use crate::utils::constants::{BUCKET_DOWNLOAD, TABLE_NAME};

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub file_id: Option<String>, 
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i32>,
    pub file_path: Option<String>,
    pub file_url: Option<String>,
    // pub dt: Option<String>,
}

impl Table {
    async fn read(ctx: SessionContext, sql: Option<&str>) -> Result<DataFrame, DataStoreError> {
        let sql = match sql {
            Some(query) => { 
                if query.contains("limit") {
                    format!("select * from {TABLE_NAME} where {query}")
                } else {
                    format!("select * from {TABLE_NAME} where {query} limit 10")
                }
            }
            None => format!("select * from {TABLE_NAME} limit 10"),
        };
        
        println!("reading data to df");
        let df = ctx.sql(&sql)
            .await
            .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;

        Ok(df)
    }

    async fn read_by_id(ctx: SessionContext, id: &str) -> Result<DataFrame, DataStoreError> {
        let sql = format!("select * from {TABLE_NAME} where file_name = '{id}'");
        println!("reading data to df");
        let df = ctx.sql(&sql)
            .await
            .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;

        Ok(df)
    }
}

impl Table {
    async fn df_to_records(df: DataFrame) -> Result<Vec<Self>, DataStoreError> {
        println!("converting df to struct");
        let mut stream = df.execute_stream().await.map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
        let mut records = vec![];
        while let Some(batch) = stream.next().await.transpose().map_err(|e| DataStoreError::UnexpectedError(e.into()))? {
            let file_ids = batch.column(0).as_string::<i32>();
            let file_names = batch.column(1).as_string::<i32>();
            let file_types = batch.column(2).as_string::<i32>();
            // let file_sizes = batch.column(2).as_primitive::<Int32Type>(); // TODO doesnot work when deserializing
            let file_paths = batch.column(4).as_string::<i32>();
            let file_urls = batch.column(5).as_string::<i32>();
            // let dts = batch.column(6).as_string::<i32>();

            for (file_id, file_name, file_type, file_path, file_url) in izip!(file_ids, file_names, file_types, file_paths, file_urls) {
                records.push(Self {
                    file_id: file_id.map(|x| x.to_string()),
                    file_name: file_name.map(|x| x.to_string()),
                    file_type: file_type.map(|x| x.to_string()),
                    file_size: None,
                    file_path: file_path.map(|x| x.to_string()),
                    file_url: file_url.map(|x| x.to_string()),
                    // dt: dt.map(|x| x.to_string()),
                });
            }
        }

        Ok(records)
    }

    pub async fn select(ctx: SessionContext, sql: Option<&str>) -> Result<Vec<Table>, DataStoreError> {
        println!("running select for table with sql: {:?}", sql);
        let df = Table::read(ctx, sql).await?;
        match is_empty(df.clone()).await {
            Ok(res) => match res {
                true => return Err(DataStoreError::QueryResultIsEmpty),
                false => {
                    let records = Self::df_to_records(df)
                        .await
                        .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                    Ok(records)
                }
            }
            Err(e) => Err(DataStoreError::UnexpectedError(e.into()))
        }
    }

    pub async fn select_by_id(ctx: SessionContext, id: &str) -> Result<Vec<Table>, DataStoreError> {
        println!("running select for table by id: {:?}", id);
        let df = Table::read_by_id(ctx, id).await?;
        match is_empty(df.clone()).await {
            Ok(res) => match res {
                true => return Err(DataStoreError::QueryResultIsEmpty),
                false => {
                    let records = Self::df_to_records(df)
                        .await
                        .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                    Ok(records)
                }
            }
            Err(e) => Err(DataStoreError::UnexpectedError(e.into()))
        }
    }

    // pub async fn download_id(id: &str) -> Result<Vec<u8>> {
    //     let df = Table::read_by_id(id).await?;
    //     let df = df.select_columns(&["file_path"])?; // #TODO change to const value
    //     let mut stream = df.execute_stream().await?;
    //     let mut file_pathes = vec![];
    //     while let Some(batch) = stream.next().await.transpose()? {
    //         let names = batch.column(0).as_string::<i32>();
    //         for name in names {
    //             let name = name.map(|x| String::from_utf8(x.into()).unwrap());
    //             file_pathes.push(name.unwrap());
    //         }
    //     }

    //     let file_path = file_pathes.get(0).unwrap();
    //     println!("start downloading file: {}", file_path);
    //     let client = get_aws_client(REGION).await;
    //     let res = read_file(client, BUCKET_DOWNLOAD.to_owned(), file_path.to_owned()).await?;

    //     Ok(res)
    // }

    pub async fn download(ctx: SessionContext, client: Client, sql: Option<&str>) -> Result<Vec<Vec<u8>>, DataStoreError> {
        let df = Table::read(ctx, sql)
            .await
            .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
        match is_empty(df.clone()).await {
            Ok(res) => match res {
                true => return Err(DataStoreError::QueryResultIsEmpty),
                false => {
                    let df = df.select_columns(&["file_path"]).map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                    let mut stream = df.execute_stream().await.map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                    let mut file_pathes = vec![];
                    while let Some(batch) = stream.next().await.transpose().map_err(|e| DataStoreError::UnexpectedError(e.into()))? {
                        let names = batch.column(0).as_string::<i32>();
                        for name in names {
                            let name = name.map(|x| String::from_utf8(x.into()).unwrap_or_default());
                            file_pathes.push(name.unwrap());
                        }
                    }
            
                    println!("start downloading files: {:?}", file_pathes);
                    let mut tasks = vec![];
                    let mut data = vec![];
                    for file_path in file_pathes {
                        let task = tokio::spawn(read_file(client.clone(), BUCKET_DOWNLOAD.to_owned(), file_path));
                        tasks.push(task);
                    }
                    for task in tasks {
                        let task = task.await.map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                        let res = task.map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
                        data.push(res);
                    }
            
                    Ok(data)
                }
            }
            Err(e) => Err(DataStoreError::UnexpectedError(e.into()))
        }
    }
}

pub async fn init(region: &str, bucket: &str, key: &str, table_name: &str) -> Result<SessionContext, DataStoreError> {
    let creds = Credentials::default().map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
    let aws_access_key_id = creds.access_key.unwrap_or_default();
    let aws_secret_access_key = creds.secret_key.unwrap_or_default();
    let aws_session_token = creds.session_token.unwrap_or_default();
    
    let s3 = AmazonS3Builder::new()
        .with_bucket_name(bucket)
        .with_region(region)
        .with_access_key_id(aws_access_key_id)
        .with_secret_access_key(aws_secret_access_key)
        .with_token(aws_session_token)
        .build()
        .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;

    let path = format!("s3://{bucket}");
    let s3_url = Url::parse(&path)
        .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;
    let ctx = SessionContext::new();
    ctx.runtime_env().register_object_store(&s3_url, Arc::new(s3));
    let path = format!("s3://{bucket}/{key}");
    ctx.register_parquet(table_name, &path, ParquetReadOptions::default())
        .await
        .map_err(|e| DataStoreError::UnexpectedError(e.into()))?;

    Ok(ctx)
}