mod bot;
mod gpt;
mod memory;
mod types;

use crate::memory::CONFIG_STORE;
use bot::handle_message;
use ic_cdk::{
    api::management_canister::http_request::{
        HttpResponse as HttpResponseCdk, TransformArgs as TransformArgsCdk,
    },
    init, query, update,
};
use memory::is_token_valid;
use telegram_bot_raw::{MessageKind, Update, UpdateKind};
use types::{Config, HeaderField, HttpRequest, HttpResponse};

#[init]
fn init(arg: Config) {
    ic_cdk::println!("canister init {}", ic_cdk::api::time());
    CONFIG_STORE.with(|config_store| {
        *config_store.borrow_mut() = arg.clone();
    });
}

#[update]
async fn http_request_update(req: HttpRequest) -> HttpResponse {
    handle_http_request(req).await
}

#[query(composite = true)]
async fn http_request(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("text/html"),
        )],
        body: "Waiting".as_bytes().to_vec(),
        upgrade: Some(true),
    }
}

#[query]
fn transform(raw: TransformArgsCdk) -> HttpResponseCdk {
    HttpResponseCdk {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
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

async fn handle_telegram(token: &str, req: HttpRequest) -> HttpResponse {
    if is_token_valid(token.to_string()) {
        match serde_json::from_slice::<Update>(&req.body) {
            Err(err) => HttpResponse {
                status_code: 500,
                headers: vec![HeaderField(
                    String::from("content-type"),
                    String::from("text/plain"),
                )],
                body: format!("{}", err).as_bytes().to_vec(),
                upgrade: Some(true),
            },
            Ok(update) => match update.kind {
                UpdateKind::Message(msg) => match msg.kind {
                    MessageKind::Text { data, .. } => {
                        handle_message(msg.from.username.unwrap(), msg.chat, data).await
                    }
                    _ => ok200(),
                },
                _ => ok200(),
            },
        }
    } else {
        HttpResponse {
            status_code: 200,
            headers: vec![HeaderField(
                String::from("content-type"),
                String::from("text/plain"),
            )],
            body: format!("Invalid Bot Token!").as_bytes().to_vec(),
            upgrade: Some(true),
        }
    }
}

fn ok200() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("text/html"),
        )],
        body: "Nothing to do".as_bytes().to_vec(),
        upgrade: Some(true),
    }
}

fn index(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
        body: format!("This is a Telegram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nFind me on telegram:\nhttps://t.me/canister_ai_bot\nFind me on browser:\nhttps://{}.raw.icp0.io/\n", ic_cdk::id(), ic_cdk::api::time(), ic_cdk::api::canister_balance(), ic_cdk::id()).as_bytes().to_vec(),
        upgrade: Some(true),
    }
}
fn err404(req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("text/plain"),
        )],
        body: format!(
            "Nothing found at {}\n(but still, you reached the internet computer!)",
            req.url
        )
        .as_bytes()
        .to_vec(),
        upgrade: Some(true),
    }
}

#[ic_cdk::query]
fn get_config() -> Config {
    CONFIG_STORE.with(|config_store| {
        config_store.borrow().clone()
    })
}

ic_cdk::export_candid!();
