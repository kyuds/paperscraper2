use aws_sdk_bedrockruntime::Client as BedrockClient;

pub struct BatchSummarizer {
    client: BedrockClient
}

impl BatchSummarizer {
    pub fn new(client: BedrockClient) -> Self {
        BatchSummarizer {
            client
        }
    }

    // pub async fn request_batch(&self, data: &Vec<ArxivResult>) {
    //     // TODO: implement
    // }
}
