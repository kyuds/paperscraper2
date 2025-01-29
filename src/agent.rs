use aws_sdk_bedrockruntime::{
    error::SdkError, 
    operation::invoke_model::InvokeModelError, 
    Client as BedrockClient
};
use aws_sdk_s3::{
    config::http::HttpResponse, 
    primitives::Blob
};
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use std::{
    error::Error as StdError,
    fmt
};

use crate::{
    model::ArxivResult,
    prompt::PROMPT
};

// we hardcode the model id as each model has different input schemas.
const MODEL_ID: &str = "us.amazon.nova-lite-v1:0";

pub struct BedrockAgent {
    client: BedrockClient
}

impl BedrockAgent {
    pub fn new(client: BedrockClient) -> Self {
        BedrockAgent {
            client
        }
    }

    pub async fn summarize(
        &self, 
        data: ArxivResult
    ) -> Result<ArxivResult, AgentError> {
        let model_input = ModelInput::default(&data.summary);
        let input = serde_json::to_string(&model_input).unwrap();

        let raw = self.client.invoke_model()
            .body(Blob::new(input))
            .content_type("application/json")
            .model_id(MODEL_ID)
            .send()
            .await
            .map_err(AgentError::from)?
            .body;
        
        let response = ModelResponse::from(raw).map_err(AgentError::from)?;
        response.combine_arxiv(data)
    }
}

// request parameters structs.

#[derive(Debug, Serialize)]
struct ModelInput {
    system: Vec<BedrockText>,
    messages: Vec<UserMessage>,
    #[serde(rename = "inferenceConfig")]
    inference_config: InferenceConfig
}

#[derive(Debug, Serialize)]
struct UserMessage {
    role: String, // "user"
    content: Vec<BedrockText>
}

#[derive(Debug, Deserialize, Serialize)]
struct BedrockText {
    text: String
}

#[derive(Debug, Serialize)]
struct InferenceConfig {
    max_new_tokens: u32, // 150
    top_p: f32, // 0.9
    top_k: u32, // 20
    temperature: f32 // 0.5
}

impl ModelInput {
    fn new(system: &str, content: &str) -> Self {
        ModelInput {
            system: vec![ BedrockText { text: system.to_string() } ],
            messages: vec![
                UserMessage {
                    role: "user".to_string(),
                    content: vec![ BedrockText { text: content.to_string() } ]
                }
            ],
            inference_config: InferenceConfig::default()
        }
    }

    fn default(content: &str) -> Self {
        Self::new(PROMPT, content)
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

// response parameter structs.

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ModelResponse {
    output: ModelOutput,
    #[serde(rename = "stopReason")]
    stop_reason: String,
    usage: ModelUsage
}

#[derive(Debug, Deserialize)]
struct ModelOutput {
    message: ModelMessage
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ModelMessage {
    content: Vec<BedrockText>,
    role: String
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ModelUsage {
    #[serde(rename = "inputTokens")]
    input_tokens: u32,
    #[serde(rename = "outputTokens")]
    output_tokens: u32,
    #[serde(rename = "totalTokens")]
    total_tokens: u32
}

impl ModelResponse {
    fn from(raw: Blob) -> Result<Self, JsonError> {
        let bytes = raw.into_inner();
        let text = String::from_utf8_lossy(&bytes);
        println!("{}", text);
        serde_json::from_str(&text)
    }

    fn get_output(self) -> String {
        self.output.message.content
            .into_iter().nth(0)
            .map(|t| { t.text })
            .unwrap_or_default()
    }

    fn combine_arxiv(self, data: ArxivResult) -> Result<ArxivResult, AgentError> {
        let summary = self.get_output();
        if summary.is_empty() {
            return Err(AgentError::new("summary is empty"));
        }
        Ok(ArxivResult {
            id: data.id,
            title: data.title,
            summary,
            authors: data.authors,
            published: data.published,
            link: data.link
        })
    }
}

#[derive(Debug)]
pub struct AgentError {
    pub message: String
}

impl AgentError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for AgentError {}

impl From<JsonError> for AgentError {
    fn from(err: JsonError) -> Self {
        AgentError::new(&format!("Json deserializing error: {}", err))
    }
}

impl From<SdkError<InvokeModelError, HttpResponse>> for AgentError {
    fn from(err: SdkError<InvokeModelError, HttpResponse>) -> Self {
        AgentError::new(&format!("AWS SDK error: {}", err))
    }
}
