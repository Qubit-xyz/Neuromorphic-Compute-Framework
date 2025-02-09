use std::env;

use rig::{
    agent::{Agent, AgentBuilder},
    cli_chatbot::cli_chatbot,
    completion::{Chat, CompletionModel, Message, PromptError},
    providers::openai::Client as OpenAIClient,
};

/// Represents a multi agent application translating text between 2 frameworks 
/// Translator agent is Qubit, translating text provided from GPT-4
struct EnglishTranslator<M: CompletionModel> {
    translator_agent: Agent<M>,
    gpt4: Agent<M>,
}

impl<M: CompletionModel> EnglishTranslator<M> {
    fn new(model: M) -> Self {
        Self {
            // Create the translator agent
            translator_agent: AgentBuilder::new(model.clone())
                .preamble("\
                    You are a translator assistant that will translate any input text into english. \
                    If the text is already in english, simply respond with the original text but fix any mistakes (grammar, syntax, etc.). \
                ")
                .build(),

            // Create the GPT4 model
            gpt4: AgentBuilder::new(model).build()
        }
    }
}

impl<M: CompletionModel> Chat for EnglishTranslator<M> {
    async fn chat(&self, prompt: &str, chat_history: Vec<Message>) -> Result<String, PromptError> {
        // Translate the prompt using the translator agent
        let translated_prompt = self
            .translator_agent
            .chat(prompt, chat_history.clone())
            .await?;

        println!("Translated prompt: {}", translated_prompt);

        // Answer the prompt using gpt4
        self.gpt4.chat(&translated_prompt, chat_history).await
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create OpenAI client
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let openai_client = OpenAIClient::new(&openai_api_key);
    let model = openai_client.completion_model("gpt-4");

    // Create OpenAI client
    // let cohere_api_key = env::var("COHERE_API_KEY").expect("COHERE_API_KEY not set");
    // let cohere_client = CohereClient::new(&cohere_api_key);
    // let model = cohere_client.completion_model("command-r");

    // Create model
    let translator = EnglishTranslator::new(model);

    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let openai_client = OpenAIClient::new(&openai_api_key);
    let model = openai_client.completion_model("gpt-4");

    // Spin up a chatbot using the agent
    cli_chatbot(translator).await?;

    Ok(())
}
