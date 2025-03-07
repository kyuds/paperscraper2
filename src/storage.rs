use std::{
    fs::File, 
    io::{self, Write}, 
    path::Path,
    error::Error as StdError,
    fmt,
    io::Error as IOError
};
use aws_sdk_s3::{
    error::SdkError, 
    operation::put_object::{PutObjectError, PutObjectOutput}, 
    primitives::ByteStream, 
    primitives::ByteStreamError,
    Client as S3Client
};
use serde_json::{self, Error as JsonError};
use uuid::Uuid;

use crate::model::ProcessedResult;

// Utils to store (temporary) files on local device.
// When using with AWS Lambda, these local files (in /tmp) will automatically be
// deleted after function invocation is complete.

struct Formatter;

impl Formatter {
    fn to_readme(data: &ProcessedResult) -> Result<String, JsonError> {
        Ok(format!("### {}\n_{}_<br/>\n{}<br/>\n_Published: {}_, [{}]({})\n\n",
            data.title,
            data.authors.join(", "),
            data.summary,
            data.published.format("%Y.%m.%d"),
            data.link, data.link
        ))
    }

    fn to_jsonl(data: &ProcessedResult) -> Result<String, JsonError> {
        let jstring = serde_json::to_string(data)?;
        Ok(format!("{}\n", jstring))
    }
}

fn save_arxiv_as_file<F>(fname: &str, op: F, data: &Vec<ProcessedResult>) -> io::Result<()>
where
    F: Fn(&ProcessedResult) -> Result<String, JsonError>
{
    let mut file = File::create(fname)?;
    data.iter()
        .filter_map(|data| {
            match op(data) {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("serde_json error: {}", e);
                    None
                }
            }
        })
        .try_for_each(|line| -> io::Result<()> {
            file.write_all(line.as_bytes())?;
            Ok(())
        })?;
    file.flush()?;
    Ok(())
}

// Utils to store data to AWS S3.
pub struct S3Storage {
    client: S3Client,
    is_lambda: bool
}

#[allow(dead_code)]
impl S3Storage {
    pub fn new(client: S3Client, is_lambda: bool) -> Self {
        S3Storage {
            client,
            is_lambda
        }
    }

    pub fn default(client: S3Client) -> Self {
        Self::new(client, true)
    }

    pub async fn upload_arxiv_as_readme(
        &self,
        bucket: &str,
        key: &str,
        data: &Vec<ProcessedResult>
    ) -> Result<PutObjectOutput, StorageError> {
        let tmp_file = self.get_fname("readme", "md");
        save_arxiv_as_file(&tmp_file, Formatter::to_readme, data)?;
        self.upload(bucket, key, &tmp_file).await
    }

    pub async fn upload_arxiv_as_jsonl(
        &self,
        bucket: &str,
        key: &str,
        data: &Vec<ProcessedResult>
    ) -> Result<PutObjectOutput, StorageError> {
        let tmp_file = self.get_fname("tmp", "jsonl");
        save_arxiv_as_file(&tmp_file, Formatter::to_jsonl, data)?;
        self.upload(bucket, key, &tmp_file).await
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        tmp_file: &str
    ) -> Result<PutObjectOutput, StorageError> {
        let input = ByteStream::from_path(Path::new(tmp_file)).await?;
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(input)
            .send()
            .await
            .map_err(StorageError::from)
    }

    fn get_fname(&self, prefix: &str, ext: &str) -> String {
        let mut fname = format!("{}_{}.{}", prefix, Uuid::new_v4(), ext);
        if self.is_lambda {
            fname = "/tmp/".to_string() + &fname;
        }
        fname
    }
}

// utils: custom error model for storage
#[derive(Debug)]
pub struct StorageError {
    pub message: String
}

impl StorageError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for StorageError {}

impl From<IOError> for StorageError {
    fn from(err: IOError) -> Self {
        StorageError::new(&format!("IO error: {}", err))
    }
}

impl From<SdkError<PutObjectError>> for StorageError {
    fn from(err: SdkError<PutObjectError>) -> Self {
        StorageError::new(&format!("AWS SDK error: {}", err))
    }
}

impl From<ByteStreamError> for StorageError {
    fn from(err: ByteStreamError) -> Self {
        StorageError::new(&format!("AWS SDK ByteStream error: {}", err))
    }
}
