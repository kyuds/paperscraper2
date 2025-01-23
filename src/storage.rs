use std::{
    fs::File, 
    io::{self, Write}, 
};
use aws_sdk_s3::{
    error::SdkError, 
    operation::put_object::{PutObjectError, PutObjectOutput}, 
    primitives::ByteStream, 
    Client
};

use crate::{
    parser::ArxivResult,
    format::Formatter
};

// Utils to store readme files on local device.
pub struct LocalSaver;

impl LocalSaver {
    pub fn save_raw_arxiv_results_as_readme(fname: &str, data: &Vec<ArxivResult>) -> io::Result<()> {
        let mut file = File::create(fname)?;
        data.iter().try_for_each(|result| -> io::Result<()> {
            file.write_all(Formatter::to_readme(result).as_bytes())?;
            Ok(())
        })?;
        file.flush()?;
        Ok(())
    }

    pub fn save_raw_arxiv_as_jsonl(fname: &str, data: &Vec<ArxivResult>) -> io::Result<()> {
        let mut file = File::create(fname)?;
        data.iter().enumerate().try_for_each(|(id, result)| -> io::Result<()> {
            file.write_all(Formatter::to_jsonl_with_id(id, result).as_bytes())?;
            Ok(())
        })?;
        Ok(())
    }
}

// Utils to store data to AWS S3.
pub struct S3Saver;

impl S3Saver {
    // pub fn save_raw_arxiv_as_jsonl(fname: &str, data: &Vec<ArxivResult>)
    // pub fn save_arxiv_bedrock_input()

    pub async fn upload_raw_arxiv_as_jsonl(
        client: &Client,
        bucket: &str,
        key: &str,
        data: &Vec<ArxivResult>
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {

        let bytes = data.iter()
            .enumerate()
            .map(|(id, arxiv)| { 
                Formatter::to_jsonl_with_id(id, arxiv) 
            })
            .collect::<Vec<_>>()
            .join("");

        Self::upload(client, bucket, key, bytes.into_bytes()).await
    }

    async fn upload(
        client: &Client,
        bucket: &str,
        key: &str,
        data: Vec<u8>
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {

        client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data))
            .send()
            .await
    }
}
