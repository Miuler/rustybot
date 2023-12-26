use std::env;

use async_openai::{Client, config::AzureConfig};
use async_openai::error;
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionRequestArgs};
use config::Config;
use dotenvy::dotenv;
use log::{debug, info};

use crate::RustybotError::{ConfigError, DotenvError, OpenAIError, VarError};

// #[derive(EnvConfig)]
// struct Config {
//     #[envconfig(from = "OPENAI_API_KEY")]
//     api_key: String,
// }

#[tokio::main]
async fn main() -> Result<(), RustybotError> {
    if let Err(e) = log4rs::init_file("log4rs.yml", Default::default()) {
        println!("No existe el log4rs.yml: {:?}", e);
    }
    let _ = dotenv();

    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;
    debug!("config: {:?}", config);

    let system_prompt: String = config.get("system_prompt")?;
    let user_prompts: Vec<String> = config.get("user_prompt")?;

    let mut prompts: Vec<ChatCompletionRequestMessage> = Vec::new();
    prompts.push(ChatCompletionRequestSystemMessageArgs::default()
        .content(system_prompt)
        .build()?
        .into());

    for user_prompt in user_prompts {
        prompts.push(ChatCompletionRequestUserMessageArgs::default()
            .content(user_prompt.clone())
            .build()?
            .into());
        info!("user_prompt: \n{}", user_prompt);

        let assistant = execute(
            CreateChatCompletionRequestArgs::default()
                .messages(prompts.clone())
                .build()?
        ).await?;

        let assistant = assistant.get(0).unwrap().clone();
        info!("assistant: \n{}", assistant);

        prompts.push(ChatCompletionRequestAssistantMessageArgs::default()
            .content(assistant)
            .build()?
            .into());
    }


    Ok(())
}

async fn execute(request: CreateChatCompletionRequest) -> Result<Vec<String>, RustybotError> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let api_base = env::var("ENDPOINT")?;
    let deployment_id = env::var("DEPLOYMENT")?;
    // debug!("api_key: {}", api_key);
    // debug!("api_base: {}", api_base);
    // debug!("deployment_id: {}", deployment_id);

    let azure_config = AzureConfig::new()
        .with_deployment_id(deployment_id)
        .with_api_base(api_base)
        .with_api_key(api_key)
        .with_api_version("2023-05-15");

    let client = Client::with_config(azure_config);

    let date1 = chrono::Utc::now();
    debug!("date1: {}", date1);
    let chat_completion_response = client.chat().create(request).await?;
    let date2 = chrono::Utc::now();
    debug!("date2: {}", date2);
    debug!("diferencia: {}", date2.signed_duration_since(date1));

    chat_completion_response.usage;

    let vec = chat_completion_response
        .choices
        .iter()
        .filter(|x| x.message.content.is_some())
        .map(|x| x.message.content.as_ref().unwrap().clone())
        .collect::<Vec<String>>();

    Ok(vec)
}


#[derive(Debug)]
enum RustybotError {
    DotenvError(dotenvy::Error),
    VarError(env::VarError),
    OpenAIError(error::OpenAIError),
    ConfigError(config::ConfigError),
}

impl From<dotenvy::Error> for RustybotError {
    fn from(e: dotenvy::Error) -> Self {
        DotenvError(e)
    }
}

impl From<env::VarError> for RustybotError {
    fn from(e: env::VarError) -> Self {
        VarError(e)
    }
}

impl From<error::OpenAIError> for RustybotError {
    fn from(e: error::OpenAIError) -> Self {
        OpenAIError(e)
    }
}

impl From<config::ConfigError> for RustybotError {
    fn from(e: config::ConfigError) -> Self {
        ConfigError(e)
    }
}