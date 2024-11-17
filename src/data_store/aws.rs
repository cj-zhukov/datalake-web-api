use anyhow::Result;
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

use crate::utils::aws::{get_aws_client, read_file};
use crate::utils::constants::{BUCKET_DOWNLOAD, BUCKET_SELECT, PREFIX_SELECT, REGION};

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
    async fn df_to_records(df: DataFrame) -> Result<Vec<Self>> {
        println!("converting df to struct");
        let mut stream = df.execute_stream().await?;
        let mut records = vec![];
        while let Some(batch) = stream.next().await.transpose()? {
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

    async fn read(sql: Option<&str>) -> Result<DataFrame> {
        let creds = Credentials::default()?;
        let aws_access_key_id = creds.access_key.unwrap_or_default();
        let aws_secret_access_key = creds.secret_key.unwrap_or_default();
        let aws_session_token = creds.session_token.unwrap_or_default();
        
        let bucket = BUCKET_SELECT.as_str();
        let region = REGION;
        let key = PREFIX_SELECT.as_str();

        let s3 = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_region(region)
            .with_access_key_id(aws_access_key_id)
            .with_secret_access_key(aws_secret_access_key)
            .with_token(aws_session_token)
            .build()?;
    
        let path = format!("s3://{bucket}");
        let s3_url = Url::parse(&path)?;
        let ctx = SessionContext::new();
        ctx.runtime_env().register_object_store(&s3_url, Arc::new(s3));
    
        let path = format!("s3://{bucket}/{key}");
        ctx.register_parquet("t", &path, ParquetReadOptions::default()).await?;
        
        let sql = match sql {
            Some(query) => { 
                if query.contains("limit") {
                    format!("select * from t where {query}")
                } else {
                    format!("select * from t where {query} limit 5")
                }
            }
            None => "select * from t limit 10".to_string(),
        };
        
        println!("reading data to df");
        let df = ctx.sql(&sql).await?;

        Ok(df)
    }

    async fn read_id(id: &str) -> Result<DataFrame> {
        let creds = Credentials::default()?;
        let aws_access_key_id = creds.access_key.unwrap_or_default();
        let aws_secret_access_key = creds.secret_key.unwrap_or_default();
        let aws_session_token = creds.session_token.unwrap_or_default();
        
        let bucket = BUCKET_SELECT.as_str();
        let region = REGION;
        let key = PREFIX_SELECT.as_str();

        let s3 = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_region(region)
            .with_access_key_id(aws_access_key_id)
            .with_secret_access_key(aws_secret_access_key)
            .with_token(aws_session_token)
            .build()?;
    
        let path = format!("s3://{bucket}");
        let s3_url = Url::parse(&path)?;
        let ctx = SessionContext::new();
        ctx.runtime_env().register_object_store(&s3_url, Arc::new(s3));
    
        let path = format!("s3://{bucket}/{key}");
        ctx.register_parquet("t", &path, ParquetReadOptions::default()).await?;
        
        let sql = format!("select * from t where file_name = '{}'", id);
        
        println!("reading data to df");
        let df = ctx.sql(&sql).await?;

        Ok(df)
    }

    pub async fn select(sql: Option<&str>) -> Result<Option<Vec<Table>>> {
        println!("running select for table with sql: {:?}", sql);
        let df = Table::read(sql).await?;
        let records = Self::df_to_records(df).await?;

        Ok(Some(records))
    }

    pub async fn select_id(id: &str) -> Result<Option<Vec<Table>>> {
        println!("running select for table with id: {:?}", id);
        let df = Table::read_id(id).await?;
        let records = Self::df_to_records(df).await?;

        Ok(Some(records))
    }

    pub async fn download_id(id: &str) -> Result<Vec<u8>> {
        let df = Table::read_id(id).await?;
        let df = df.select_columns(&["file_path"])?; // #TODO change to const value
        let mut stream = df.execute_stream().await?;
        let mut file_pathes = vec![];
        while let Some(batch) = stream.next().await.transpose()? {
            let names = batch.column(0).as_string::<i32>();
            for name in names {
                let name = name.map(|x| String::from_utf8(x.into()).unwrap());
                file_pathes.push(name.unwrap());
            }
        }

        let file_path = file_pathes.get(0).unwrap();
        println!("start downloading file: {}", file_path);
        let client = get_aws_client(REGION).await;
        let res = read_file(client, BUCKET_DOWNLOAD.to_owned(), file_path.to_owned()).await?;

        Ok(res)
    }

    pub async fn download(sql: Option<&str>) -> Result<Vec<Vec<u8>>> {
        let df = Table::read(sql).await?;
        let df = df.select_columns(&["file_path"])?; // #TODO change to const value
        let mut stream = df.execute_stream().await?;
        let mut file_pathes = vec![];
        while let Some(batch) = stream.next().await.transpose()? {
            let names = batch.column(0).as_string::<i32>();
            for name in names {
                let name = name.map(|x| String::from_utf8(x.into()).unwrap());
                file_pathes.push(name.unwrap());
            }
        }

        println!("start downloading files: {:?}", file_pathes);
        let mut tasks = vec![];
        let mut data = vec![];
        let client = get_aws_client(REGION).await;
        for file_path in file_pathes {
            let task = tokio::spawn(read_file(client.clone(), BUCKET_DOWNLOAD.to_owned(), file_path));
            tasks.push(task);
        }
        for task in tasks {
            let res = task.await??;
            data.push(res);
        }

        Ok(data)
    }

    // pub async fn download(sql: Option<&str>) -> Result<Vec<u8>> {
    //     let df = Table::read(sql).await?;
    //     let df = df.select_columns(&["file_path"])?; // TODO change to const value
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
}

