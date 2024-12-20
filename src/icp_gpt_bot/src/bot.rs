
use crate::types::{HttpResponse, HeaderField, TransformArgs};
use serde_json::{Value, json};
use telegram_bot_raw::{MessageChat, SendMessage, SendChatAction, ChatAction};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse as HttpResponseCdk, 
    TransformContext as TransformContextCdk, http_request, TransformArgs as TransformArgsCdk, 
};
use candid::Nat;


pub async fn handle_message(chat: MessageChat, text: String) -> HttpResponse {
    match text.as_str() {
        "/start" => {
            send_message(
                chat,
                "Hello! I am a Telegram Bot on Internet Computer using ChatGPT.\nTry /info to get my information.\nChat and Image generation coming soon.\n".to_string(),
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
        _ => {
            match text.as_str().strip_prefix("/chat ") {
                Some(prompt) => {
                    ic_cdk::println!("fdgdfg");
                    let reply = call_chatgpt(prompt.to_string()).await;
                    send_message(
                        chat.clone(),
                        reply,
                    )
                }
                None => send_message(chat, "Invalid Prompt".to_string()),
            }
        },
    }
}

pub async fn call_chatgpt(prompt: String) -> String {
    let url = "https://us-central1-telegram-gpt-488cd.cloudfunctions.net/chatgpt/chat".to_string();

    let body = json!({
        "prompt": prompt,
        "date": ic_cdk::api::time().to_string()
    })
    .to_string();

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(50_000), // 50 KB
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


fn send_chat_action(chat: MessageChat) -> HttpResponse {
    let m = SendChatAction::new(chat, ChatAction::Typing);
    let value = serde_json::to_value(m).unwrap();
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("application/json"))],
        body: serde_json::to_vec(&value).unwrap(),
        upgrade: Some(true),
    }
}

fn add_method(value: &mut Value, method: String) {
    match value {
        Value::Object(m) => {
            m.insert("method".to_string(), Value::String(method));
        }
        _ => (),
    }
}

pub fn transform_response(raw: TransformArgsCdk) -> HttpResponseCdk {
    // let headers = vec![
    //     HttpHeader {
    //         name: "Content-Security-Policy".to_string(),
    //         value: "default-src 'self'".to_string(),
    //     },
    //     HttpHeader {
    //         name: "Referrer-Policy".to_string(),
    //         value: "strict-origin".to_string(),
    //     },
    //     HttpHeader {
    //         name: "Permissions-Policy".to_string(),
    //         value: "geolocation=(self)".to_string(),
    //     },
    //     HttpHeader {
    //         name: "Strict-Transport-Security".to_string(),
    //         value: "max-age=63072000".to_string(),
    //     },
    //     HttpHeader {
    //         name: "X-Frame-Options".to_string(),
    //         value: "DENY".to_string(),
    //     },
    //     HttpHeader {
    //         name: "X-Content-Type-Options".to_string(),
    //         value: "nosniff".to_string(),
    //     },
    // ];

    HttpResponseCdk {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
}