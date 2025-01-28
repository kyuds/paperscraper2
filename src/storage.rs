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
use serde::Serialize;
use uuid::Uuid;

use crate::{
    model::ArxivResult,
    prompt::PROMPT
};

// Utils to store (temporary) files on local device.
// When using with AWS Lambda, these local files (in /tmp) will automatically be
// deleted after function invocation is complete.

struct Formatter;

impl Formatter {
    fn to_readme(data: &ArxivResult) -> Result<String, JsonError> {
        Ok(format!("### {}\n_{}_<br/>\n{}<br/>\n_Published: {}_, [{}]({})\n\n",
            data.title,
            data.authors.join(", "),
            data.summary,
            data.published.format("%Y.%m.%d"),
            data.link, data.link
        ))
    }

    fn to_jsonl(data: &ArxivResult) -> Result<String, JsonError> {
        let jstring = serde_json::to_string(data)?;
        Ok(format!("{}\n", jstring))
    }

    fn to_bedrock_input(data: &ArxivResult) -> Result<String, JsonError> {
        let batch_request = BatchRequest::new(data);
        let jstring = serde_json::to_string(&batch_request)?;
        Ok(format!("{}\n", jstring))
    }
}

fn save_arxiv_as_file<F>(fname: &str, op: F, data: &Vec<ArxivResult>) -> io::Result<()>
where
    F: Fn(&ArxivResult) -> Result<String, JsonError>
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

    pub async fn upload_raw_arxiv_as_readme(
        &self,
        bucket: &str,
        key: &str,
        data: &Vec<ArxivResult>
    ) -> Result<PutObjectOutput, StorageError> {
        let tmp_file = self.get_fname("readme", "md");
        save_arxiv_as_file(&tmp_file, Formatter::to_readme, data)?;
        self.upload(bucket, key, &tmp_file).await
    }

    pub async fn upload_raw_arxiv_as_jsonl(
        &self,
        bucket: &str,
        key: &str,
        data: &Vec<ArxivResult>
    ) -> Result<PutObjectOutput, StorageError> {
        let tmp_file = self.get_fname("raw", "jsonl");
        save_arxiv_as_file(&tmp_file, Formatter::to_jsonl, data)?;
        self.upload(bucket, key, &tmp_file).await
    }

    pub async fn upload_bedrock_inputs(
        &self,
        bucket: &str,
        key: &str,
        data: &Vec<ArxivResult>
    ) -> Result<PutObjectOutput, StorageError> {
        let tmp_file = self.get_fname("bedrock", "jsonl");
        save_arxiv_as_file(&tmp_file, Formatter::to_bedrock_input, data)?;
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

// utils: batch invocation data format with serde, specifically for Amazon Nova Lite

#[derive(Debug, Serialize)]
struct BatchRequest {
    #[serde(rename = "recordId")]
    record_id: String,
    #[serde(rename = "modelInput")]
    model_input: ModelInput
}

#[derive(Debug, Serialize)]
struct ModelInput {
    #[serde(rename = "schemaVersion")]
    schema_version: String, // "messages-v1"
    messages: Vec<UserPrompts>,
    system: Vec<BedrockContent>,
    #[serde(rename = "inferenceConfig")]
    inference_config: InferenceConfig
}

#[derive(Debug, Serialize)]
struct UserPrompts {
    role: String, // "user"
    content: Vec<BedrockContent>
}

#[derive(Debug, Serialize)]
struct BedrockContent {
    text: String
}

#[derive(Debug, Serialize)]
struct InferenceConfig {
    max_new_tokens: u32, // 150
    top_p: f32, // 0.9
    top_k: u32, // 20
    temperature: f32 // 0.5
}

impl BatchRequest {
    fn new(data: &ArxivResult) -> Self {
        BatchRequest {
            record_id: data.record_id(),
            model_input: ModelInput::new(PROMPT, &data.summary)
        }
    }
}

impl ModelInput {
    fn new(system: &str, content: &str) -> Self {
        ModelInput {
            schema_version: "messages-v1".to_string(),
            messages: vec![UserPrompts::new(content.to_string())],
            system: vec![BedrockContent::new(system.to_string())],
            inference_config: InferenceConfig::default()
        }
    }
}

impl UserPrompts {
    fn new(content: String) -> Self {
        UserPrompts {
            role: "user".to_string(),
            content: vec![BedrockContent::new(content)]
        }
    }
}

impl BedrockContent {
    fn new(text: String) -> Self {
        BedrockContent {
            text
        }
    }
}

impl InferenceConfig {
    fn default() -> Self {
        InferenceConfig {
            max_new_tokens: 150,
            top_p: 0.9,
            top_k: 20,
            temperature: 0.5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};

    fn get_sample_arxiv() -> ArxivResult {
        ArxivResult {
            id: 0,
            title: "title".to_string(),
            summary: "summary".to_string(),
            authors: vec!["john doe".to_string()],
            published: Utc.timestamp_opt(0, 0).unwrap(),
            link: "www.example.com".to_string()
        }
    }

    const BASE_README: &str = concat!(
        "### title\n_john doe_<br/>\nsummary<br/>\n_Published: 1970.01.01_, ",
        "[www.example.com](www.example.com)\n\n"
    );

    const BASE_JSONL: &str = concat!(
        "{\"id\":0,\"title\":\"title\",\"summary\":\"summary\",\"authors\":[\"john doe\"],",
        "\"published\":\"1970-01-01T00:00:00Z\",\"link\":\"www.example.com\"}\n"
    );

    #[test]
    fn test_readme() {
        let base = String::from(BASE_README);
        let readme = Formatter::to_readme(&get_sample_arxiv()).unwrap();
        assert_eq!(base, readme);
    }

    #[test]
    fn test_jsonl() {
        let base = String::from(BASE_JSONL);
        let jsonl = Formatter::to_jsonl(&get_sample_arxiv()).unwrap();
        assert_eq!(base, jsonl);
    }
}
