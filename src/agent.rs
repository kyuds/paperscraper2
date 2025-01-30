use async_openai::{
    config::OpenAIConfig, 
    error::OpenAIError, 
    types::{
        ChatCompletionRequestSystemMessageArgs, 
        ChatCompletionRequestUserMessageArgs, 
        CreateChatCompletionRequestArgs
    }, 
    Client as OpenAIClient
};
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
    fmt, 
    sync::Arc
};
use tokio::task;

use crate::{
    model::ArxivResult,
    prompt::PROMPT
};

const OPENAI_MODEL: &str = "gpt-4o-mini";

pub struct OpenAIAgent {
    internal: Arc<OpenAIAgentInternal>
}

impl OpenAIAgent {
    pub fn new(client: OpenAIClient<OpenAIConfig>) -> Self {
        OpenAIAgent {
            internal: Arc::new(OpenAIAgentInternal::new(client))
        }
    }

    pub async fn summarize(&self, data: Vec<ArxivResult>) -> Vec<ArxivResult> {
        let internal_clone = Arc::clone(&self.internal);
        internal_clone.concurrent_summarize(data).await
    }
}

struct OpenAIAgentInternal {
    client: OpenAIClient<OpenAIConfig>
}

impl OpenAIAgentInternal {
    pub fn new(client: OpenAIClient<OpenAIConfig>) -> Self {
        OpenAIAgentInternal {
            client
        }
    }

    async fn single_summarize(
        &self, 
        mut data: ArxivResult
    ) -> Result<ArxivResult, AgentError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(OPENAI_MODEL)
            .max_tokens(150_u32)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(PROMPT)
                    .build()
                    .unwrap()
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(data.summary.as_str())
                    .build()
                    .unwrap()
                    .into(),
            ])
            .build()
            .unwrap();

        let summary = self.client
            .chat()
            .create(request)
            .await
            .map_err(AgentError::from)?
            .choices
            .into_iter()
            .next()
            .ok_or(AgentError::new("No completion"))?
            .message
            .content
            .ok_or(AgentError::new("No completion"))?;
        
        data.summary = summary;
        Ok(data)
    }

    async fn concurrent_summarize(
        self: Arc<Self>,
        data: Vec<ArxivResult>
    ) -> Vec<ArxivResult> {
        let handles = data.into_iter()
            .map(|data| { 
                let self_clone = Arc::clone(&self);
                task::spawn(async move {
                    self_clone.single_summarize(data).await
                }) 
            })
            .collect::<Vec<_>>();
        
        let mut results: Vec<ArxivResult> = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => eprintln!("Agent error: {}", e),
                Err(e) => eprintln!("Join error: {}", e)
            }
        }
        results
    }
}

// we hardcode the model id as each model has different input schemas.
const BEDROCK_MODEL_ID: &str = "us.amazon.nova-lite-v1:0";

pub struct BedrockAgent {
    internal: Arc<BedrockAgentInternal>
}

impl BedrockAgent {
    pub fn new(client: BedrockClient) -> Self {
        BedrockAgent {
            internal: Arc::new(BedrockAgentInternal::new(client))
        }
    }

    pub async fn summarize(&self, data: Vec<ArxivResult>) -> Vec<ArxivResult> {
        let internal_clone = Arc::clone(&self.internal);
        internal_clone.concurrent_summarize(data).await
    }
}

struct BedrockAgentInternal {
    client: BedrockClient
}

impl BedrockAgentInternal {
    fn new(client: BedrockClient) -> Self {
        BedrockAgentInternal {
            client
        }
    }

    async fn single_summarize(
        &self, 
        data: ArxivResult
    ) -> Result<ArxivResult, AgentError> {
        let model_input = ModelInput::default(&data.summary);
        let input = serde_json::to_string(&model_input).unwrap();

        let raw = self.client.invoke_model()
            .body(Blob::new(input))
            .content_type("application/json")
            .model_id(BEDROCK_MODEL_ID)
            .send()
            .await
            .map_err(AgentError::from)?
            .body;
        
        let response = ModelResponse::from(raw).map_err(AgentError::from)?;
        response.combine_arxiv(data)
    }

    async fn concurrent_summarize(
        self: Arc<Self>,
        data: Vec<ArxivResult>
    ) -> Vec<ArxivResult> {
        let handles = data.into_iter()
            .map(|data| { 
                let self_clone = Arc::clone(&self);
                task::spawn(async move {
                    self_clone.single_summarize(data).await
                }) 
            })
            .collect::<Vec<_>>();
        
        let mut results: Vec<ArxivResult> = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => eprintln!("Agent error: {}", e),
                Err(e) => eprintln!("Join error: {}", e)
            }
        }
        results
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
        AgentError::new(&format!("AWS SDK error: {}. Details: {:?}", err, err.raw_response()))
    }
}

impl From<OpenAIError> for AgentError {
    fn from(err: OpenAIError) -> Self {
        AgentError::new(&format!("Open AI Error: {}", err))
    }
}
