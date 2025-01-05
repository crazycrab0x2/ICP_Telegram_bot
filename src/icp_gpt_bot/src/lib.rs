mod types;
mod bot;
mod gpt;
mod memory;

use bot::handle_message;
use types::{HttpRequest, HttpResponse, HeaderField, InitArg};
use telegram_bot_raw::{MessageKind, Update, UpdateKind};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse as HttpResponseCdk, TransformArgs as TransformArgsCdk}, query, update, init
};
use crate::memory::{PROMPT_STORE, ADMIN_STORE, TOKEN_STORE, USERNAME_STORE};

// #[init]
// fn init(arg: InitArg) {
//     ADMIN_STORE.with(|admin_store| {
//         *admin_store.borrow_mut() = arg.admin;
//     });
//     TOKEN_STORE.with(|token_store| {
//         *token_store.borrow_mut() = arg.token;
//     });
//     USERNAME_STORE.with(|username_store| {
//         let mut binding = username_store.borrow_mut();
//         for username in arg.usernames {
//             binding.push(username);
//         };
//     });
//     PROMPT_STORE.with(|prompt_store| {
//         let mut binding = prompt_store.borrow_mut();
//         for prompt in arg.prompts {
//             binding.insert(prompt.shortcut, prompt.prompt);
//         }
//     })
// }

#[update]
async fn http_request_update(req: HttpRequest) -> HttpResponse {
    handle_http_request(req).await
}

#[query(composite = true)]
async fn http_request(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/html"))],
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

async fn handle_telegram(_token: &str, req: HttpRequest) -> HttpResponse {
    match serde_json::from_slice::<Update>(&req.body) {
        Err(err) => HttpResponse {
            status_code: 500,
            headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
            body: format!("{}", err).as_bytes().to_vec(),
            upgrade: Some(true),
        },
        Ok(update) => match update.kind {
            UpdateKind::Message(msg) => match msg.kind {
                MessageKind::Text { data, .. } => handle_message(msg.from.username.unwrap(), msg.chat, data).await,
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
        headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
        body: format!(
            "Nothing found at {}\n(but still, you reached the internet computer!)",
            req.url
        )
        .as_bytes()
        .to_vec(),
        upgrade: Some(true),
    }
}

ic_cdk::export_candid!();