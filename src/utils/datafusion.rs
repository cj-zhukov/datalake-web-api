use color_eyre::eyre::Result;
use datafusion::prelude::*;

pub async fn is_empty(df: DataFrame) -> Result<bool> {
    let batches = df.collect().await?;
    let is_empty = batches.iter().all(|batch| batch.num_rows() == 0);
    
    Ok(is_empty)
}
