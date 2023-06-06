use colored::Colorize;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::git::Git;
use crate::query_params::build_query;
use crate::utils;

#[derive(Debug, Deserialize, Serialize)]
struct OpenApiResponseBody {
    id: String,
    object: String,
    created: u64,
    choices: Vec<OpenApiResponseBodyChoice>,
    usage: OpenApiResponseBodyUsage,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenApiResponseBodyChoice {
    index: i32,
    message: OpenApiMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenApiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenApiResponseBodyUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenApiRequestBody {
    model: String,
    messages: Vec<OpenApiMessage>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize, Serialize)]
struct Error {
    message: Option<String>,
    #[serde(rename = "type")]
    _type: Option<String>,
    param: Option<String>,
    code: Option<String>,
}

async fn post_api_call(
    url: &str,
    body: &str,
    additional_headers: Option<HeaderMap>,
) -> Result<String, String> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Some(additional_headers) = additional_headers {
        headers.extend(additional_headers);
    }
    let res = client
        .post(url)
        .headers(headers)
        .body(body.to_owned())
        .send()
        .await;

    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }

    let res = res.unwrap().text().await;

    if res.is_err() {
        return Err(res.unwrap_err().to_string());
    }

    Ok(res.unwrap())
}

pub async fn query(
    previous_response: Option<Vec<String>>,
    git: &Git,
    files: Vec<String>,
) -> Result<String, String> {
    let config = utils::get_config();

    let mut messages: Vec<OpenApiMessage> = Vec::new();
    messages.push(OpenApiMessage {
        role: "user".to_owned(),
        content: build_query(git, files).to_owned(),
    });
    if let Some(previous_response) = previous_response {
        for response in previous_response {
            messages.push(OpenApiMessage {
                role: "agent".to_owned(),
                content: response,
            });
        }
    }
    let body = OpenApiRequestBody {
        model: "gpt-3.5-turbo".to_owned(),
        messages,
        temperature: 0.9,
        max_tokens: 150,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", config.get_api_key()).parse().unwrap(),
    );

    let result = post_api_call(
        "https://api.openai.com/v1/chat/completions",
        serde_json::to_string(&body).unwrap().as_str(),
        Some(headers),
    )
    .await;

    if let Err(err) = result {
        println!("Error: {}", err.red());
        std::process::exit(0);
    }

    let json =
        serde_json::from_str::<OpenApiResponseBody>(&result.clone().unwrap()).or_else(|_| {
            let json = serde_json::from_str::<ErrorResponse>(&result.unwrap());
            if let Err(err) = json {
                return Err(format!("{}", err));
            }
            let error = json.unwrap().error;
            let message = error.message.unwrap_or("".to_owned());
            let code = error.code.unwrap_or("".to_owned());
            println!("Error: {} {}", message.red(), code.red());
            std::process::exit(0);
        });

    if let Err(err) = json {
        return Err(format!("{}", err));
    }

    Ok(json.unwrap().choices[0].message.content.to_owned())
}
