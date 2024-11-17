use color_eyre::Result;
use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};

pub async fn zip_data(files: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    println!("processing files into zip");
    let mut zip_buffer = vec![];
    let mut zip = ZipFileWriter::new(&mut zip_buffer);
    for (index, file_data) in files.iter().enumerate() {
        let file_name = format!("file-{}", index + 1);
        let builder = ZipEntryBuilder::new(file_name.into(), Compression::Stored);
        zip.write_entry_whole(builder, file_data).await?;
    }
    zip.close().await?;

    Ok(zip_buffer)
}