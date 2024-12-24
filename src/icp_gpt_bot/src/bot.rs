
use crate::types::{HttpResponse, HeaderField};
use serde_json::{Value, json};
use telegram_bot_raw::{MessageChat, SendMessage};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse as HttpResponseCdk, 
    TransformContext as TransformContextCdk, http_request, TransformArgs as TransformArgsCdk, 
};


pub async fn handle_message(chat: MessageChat, text: String) -> HttpResponse {
    match text.as_str() {
        "/start" => {
            send_message(
                chat,
                "Hello! I am a Telegram Bot on Internet Computer using ChatGPT.\nTry /info to get my information.\nTry /chat+prompt for chat completion\nTry /imagine+prompt for image generation.\n".to_string(),
            )
        }
        "/info" => {
            send_message(
                chat,
                format!(
                    "This is a Telegram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nFind me on telegram:\nhttps://t.me/canister_ai_bot\nFind me on browser:\nhttps://{}.raw.icp0.io/\n",
                    ic_cdk::id(),
                    ic_cdk::api::time(),
                    ic_cdk::api::canister_balance(),
                    ic_cdk::id()
                ),
            )
        }
        "/chat" => {
            send_message(
                chat,
                "Send prompt after /chat\nLike /chat Hello".to_string(),
            )
        }
        "/imagine" => {
            send_message(
                chat,
                "Send prompt after /imagine\nLike /imagine a cute cat".to_string(),
            )
        }
        other => {
            if other.contains("/chat") {
                let prompt = other.strip_prefix("/chat").unwrap();
                let reply = call_chatgpt_chat(prompt.to_string()).await;
                ic_cdk::println!("chat {}", reply.clone());
                
                send_message(
                    chat.clone(),
                    reply,
                )
            }
            else if other.contains("/imagine") {
                let prompt = other.strip_prefix("/imagine").unwrap();
                // ic_cdk::println!("chat {}", prompt);
                let reply = call_chatgpt_image(prompt.to_string()).await;
                send_message(
                    chat.clone(),
                    reply,
                )
            }
            else {
                send_message(
                    chat.clone(),
                    "invalid Prompt".to_string(),
                )
            }
        },
    }
}

pub async fn call_chatgpt_chat(prompt: String) -> String {
    let url = "https://us-central1-telegram-gpt-488cd.cloudfunctions.net/chatgpt/chat".to_string();

    let body = json!({
        "prompt": prompt,
        "date": ic_cdk::api::time().to_string()
    })
    .to_string();

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "Idempotency-Key".to_string(),
                value: ic_cdk::api::time().to_string(),
            }
        ],
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(50_000), // 3MB
        transform: Some(TransformContextCdk::from_name(
            "transform".to_string(),
            vec![],
        )),
    };

    let cycles = 700_000_000;

    match http_request(request, cycles).await {
        Ok((response,)) => {
            let parse_res = String::from_utf8(response.body);
            match parse_res {
                Ok(res) => {
                    (&res[1..res.len() - 1]).to_string()
                },
                Err(_err) => {
                    "Failed to parse response".to_string()
                }  
            }
        }
        Err((r, m)) => {
            format!("HTTP request failed with code {:?}: {}", r, m)
        }
    }
}


pub async fn call_chatgpt_image(prompt: String) -> String {
    let url = "https://us-central1-telegram-gpt-488cd.cloudfunctions.net/chatgpt/image".to_string();

    let body = json!({
        "prompt": prompt,
        "date": ic_cdk::api::time().to_string()
    })
    .to_string();

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "Idempotency-Key".to_string(),
                value: ic_cdk::api::time().to_string(),
            }
        ],
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(50_000), // 3MB
        transform: Some(TransformContextCdk::from_name(
            "transform".to_string(),
            vec![],
        )),
    };

    let cycles = 700_000_000;

    match http_request(request, cycles).await {
        Ok((response,)) => {
            String::from_utf8(response.body).unwrap_or_else(|_| "Failed to parse response".to_string())
        }
        Err((r, m)) => {
            format!("HTTP request failed with code {:?}: {}", r, m)
        }
    }
}

fn send_message(chat: MessageChat, text: String) -> HttpResponse {
    let m = SendMessage::new(chat, text);
    let mut value = serde_json::to_value(m).unwrap();
    add_method(&mut value, "sendMessage".to_string());
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("application/json"),
        )],
        body: serde_json::to_vec(&value).unwrap(),
        upgrade: Some(false),
    }
}


// fn send_chat_action(chat: MessageChat) -> HttpResponse {
//     let m = SendChatAction::new(chat, ChatAction::Typing);
//     let value = serde_json::to_value(m).unwrap();
//     HttpResponse {
//         status_code: 200,
//         headers: vec![HeaderField(String::from("content-type"), String::from("application/json"))],
//         body: serde_json::to_vec(&value).unwrap(),
//         upgrade: Some(true),
//     }
// }

fn add_method(value: &mut Value, method: String) {
    match value {
        Value::Object(m) => {
            m.insert("method".to_string(), Value::String(method));
        }
        _ => (),
    }
}

pub fn transform_response(raw: TransformArgsCdk) -> HttpResponseCdk {
    HttpResponseCdk {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
}