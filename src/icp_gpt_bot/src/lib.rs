mod types;
mod bot;

use bot::{handle_message, call_chatgpt, transform_response};
use types::{HttpRequest, HttpResponse, HeaderField, TransformArgs};
use telegram_bot_raw::{MessageKind, Update, UpdateKind};
use ic_cdk::{
    api::management_canister::http_request::{HttpHeader, HttpResponse as HttpResponseCdk, TransformArgs as TransformArgsCdk}, query, update
};
use candid::Nat;


#[update]
async fn http_request_update(req: HttpRequest) -> HttpResponse {
    handle_http_request(req).await
}

#[query]
async fn http_request(req: HttpRequest) -> HttpResponse {
    handle_http_request(req).await
}

#[update]
async fn call_chatgpt_api(prompt: String) -> String {
    call_chatgpt(prompt).await
}

#[query]
fn transform(raw: TransformArgsCdk) -> HttpResponseCdk {
    transform_response(raw)
}

pub async fn handle_http_request(req: HttpRequest) -> HttpResponse {
    let uri = req.url.clone();
    match uri.strip_prefix("/webhook/") {
        Some(token) => handle_telegram(token, req).await,
        None => {
            if req.url == "/" {
                index(req)
            } else {
                err404(req)
            }
        }
    }
}

async fn handle_telegram(_token: &str, req: HttpRequest) -> HttpResponse {
    match serde_json::from_slice::<Update>(&req.body) {
        Err(err) => HttpResponse {
            status_code: 500,
            headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
            body: format!("{}", err).as_bytes().to_vec(),
            upgrade: Some(false),
        },
        Ok(update) => match update.kind {
            UpdateKind::Message(msg) => match msg.kind {
                MessageKind::Text { data, .. } => handle_message(msg.chat, data).await,
                _ => ok200(),
            },
            _ => ok200(),
        },
    }
}

fn ok200() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/html"))],
        body: "Nothing to do".as_bytes().to_vec(),
        upgrade: Some(false),
    }
}

fn index(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
        body: format!("This is a Telegram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nFind me on telegram:\nhttps://t.me/canister_ai_bot\nFind me on browser:\nhttps://{}.raw.icp0.io/\n", ic_cdk::id(), ic_cdk::api::time(), ic_cdk::api::canister_balance(), ic_cdk::id()).as_bytes().to_vec(),
        upgrade: Some(false),
    }
}
fn err404(req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
        body: format!(
            "Nothing found at {}\n(but still, you reached the internet computer!)",
            req.url
        )
        .as_bytes()
        .to_vec(),
        upgrade: Some(false),
    }
}