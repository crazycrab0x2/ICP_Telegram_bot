use crate::gpt::call_chatgpt;
use crate::types::{Form, Message, MessageType};
use crate::{
    memory::{add_new_messages, get_followed_messages, get_latest_messages},
    types::{HeaderField, HttpResponse},
};
use regex::Regex;
use serde_json::json;
use serde_json::Value;
use telegram_bot_raw::{MessageChat, SendMessage, ParseMode};

pub async fn handle_message(username: String, chat: MessageChat, text: String) -> HttpResponse {
    let timestamp = ic_cdk::api::time();

    let response = if text.contains("/") {
        if text == "/start".to_string() {
            "'Hello! I am a Telegram Bot on Internet Computer using ChatGPT.\nTry /help to get my information.\nTry to send prompt for chat completion\nTry /imagine+prompt for image generation.\n'".to_string()
        } else if text == "/help".to_string() {
            format!(
                "'This is a Telegram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nFind me on telegram:\nhttps://t.me/canister_ai_bot\nFind me on browser:\nhttps://{}.raw.icp0.io/\n'",
                ic_cdk::id(),
                timestamp,
                ic_cdk::api::canister_balance(),
                ic_cdk::id()
            )
        } else if text == "/retry".to_string() {
            core_action(MessageType::Chat, username, "".to_string(), false, true).await
        } else if text == "/imagine".to_string() {
            "'Send prompt after /Imagine\nLike /imagine a cute cat'".to_string()
        } else if text.contains("/imagine") {
            let prompt = text.strip_prefix("/imagine").unwrap();
            core_action(
                MessageType::Image,
                username,
                prompt.to_string(),
                false,
                false,
            )
            .await
        } else {
            "'Invalid Command.'".to_string()
        }
    } else {
        ic_cdk::println! {"{}", username};
        let is_follow = if text.starts_with('+') { true } else { false };
        core_action(MessageType::Chat, username, text, is_follow, false).await
    };
    send_message(chat, response[1..response.len() - 1].to_string())
}

pub async fn core_action(
    types: MessageType,
    username: String,
    prompt: String,
    is_follow: bool,
    is_retry: bool,
) -> String {
    let timestamp = ic_cdk::api::time();
    let _latest_message = get_latest_messages(username.clone());
    let followed_message = get_followed_messages(username.clone());

    let (uri, request_body, key) = if is_retry {
        if _latest_message.is_none() {
            return "There is not a previous message.".to_string();
        } else {
            let latest_message = _latest_message.unwrap();
            let key = format!(
                "{:#?}-{}-{}",
                latest_message.types, latest_message.question, timestamp
            );
            if latest_message.types == MessageType::Image {
                // retry for image generation
                let request_body = json!({
                    "model": "dall-e-3",
                    "prompt": latest_message.question,
                    "n": 1,
                })
                .to_string();
                ("image", request_body, key)
            } else {
                //retry for chat completion
                let request_body = make_chat_request(followed_message, is_retry, prompt.clone());
                ("chat", request_body, key)
            }
        }
    } else {
        if types == MessageType::Image {
            let key = format!("{:#?}-{}-{}", types, prompt.clone(), timestamp);
            let request_body = json!({
                "model": "dall-e-3",
                "prompt": prompt.clone(),
                "n": 1,
            })
            .to_string();
            ("image", request_body, key)
        } else {
            let key = format!("{:#?}-{}-{}", types, prompt.clone(), timestamp);
            let request_body = if is_follow {
                make_chat_request(followed_message, is_retry, prompt.clone())
            } else {
                make_chat_request(vec![], is_retry, prompt.clone())
            };
            ("chat", request_body, key)
        }
    };
    let mut reply = call_chatgpt(uri, request_body.clone(), key.clone()).await;
    if reply == "Rate exceeded.".to_string() {
        reply = call_chatgpt("image", request_body, key.clone()).await;
    }
    add_new_messages(
        key,
        username,
        types,
        timestamp,
        prompt,
        reply.clone(),
        is_follow,
    );
    ic_cdk::println!("before - {}", reply);
    let response = convert_to_telegram_format(&reply, "html");
    ic_cdk::println!("after - {}", response);
    response
}

fn make_chat_request(old_messages: Vec<Message>, is_retry: bool, prompt: String) -> String {
    let mut messages = vec![Form {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    }];

    old_messages
        .iter()
        .enumerate()
        .for_each(|(index, message)| {
            messages.push(Form {
                role: "user".to_string(),
                content: message.question.clone(),
            });
            if index < old_messages.len() - 1 || !is_retry {
                messages.push(Form {
                    role: "assistant".to_string(),
                    content: message.answer.clone(),
                });
            }
        });
    if !is_retry {
        messages.push(Form {
            role: "user".to_string(),
            content: prompt,
        });
    }

    json!({
        "model": "gpt-4o",
        "messages": messages
    })
    .to_string()
}

fn convert_to_telegram_format(input: &str, format_type: &str) -> String {
    let mut formatted_text = input.to_string();

    // Bold formatting - '**text**' for Markdown or <b>text</b> for HTML
    formatted_text = if format_type == "markdown" {
        let bold_re = Regex::new(r"\*\*(.*?)\*\*").unwrap();
        bold_re.replace_all(&formatted_text, r"\*\*$1\*\*").to_string()
    } else {
        let bold_re = Regex::new(r"\*\*(.*?)\*\*").unwrap();
        bold_re.replace_all(&formatted_text, r"<b>$1</b>").to_string()
    };

    // Italic formatting - '*text*' for Markdown or <i>text</i> for HTML
    formatted_text = if format_type == "markdown" {
        let italic_re = Regex::new(r"\*(.*?)\*").unwrap();
        italic_re.replace_all(&formatted_text, r"* $1 *").to_string()
    } else {
        let italic_re = Regex::new(r"\*(.*?)\*").unwrap();
        italic_re.replace_all(&formatted_text, r"<i>$1</i>").to_string()
    };

    // Monospace formatting - '`text`' for Markdown or <code>text</code> for HTML
    formatted_text = if format_type == "markdown" {
        let monospace_re = Regex::new(r"`(.*?)`").unwrap();
        monospace_re.replace_all(&formatted_text, r"`$1`").to_string()
    } else {
        let monospace_re = Regex::new(r"`(.*?)`").unwrap();
        monospace_re.replace_all(&formatted_text, r"<code>$1</code>").to_string()
    };

    // Underline formatting - '_text_' for Markdown or <u>text</u> for HTML
    formatted_text = if format_type == "markdown" {
        let underline_re = Regex::new(r"_(.*?)_").unwrap();
        underline_re.replace_all(&formatted_text, r"_$1_").to_string()
    } else {
        let underline_re = Regex::new(r"_(.*?)_").unwrap();
        underline_re.replace_all(&formatted_text, r"<u>$1</u>").to_string()
    };

    // Strikethrough formatting - '~~text~~' for Markdown or <del>text</del> for HTML
    formatted_text = if format_type == "markdown" {
        let strikethrough_re = Regex::new(r"~~(.*?)~~").unwrap();
        strikethrough_re.replace_all(&formatted_text, r"~~$1~~").to_string()
    } else {
        let strikethrough_re = Regex::new(r"~~(.*?)~~").unwrap();
        strikethrough_re.replace_all(&formatted_text, r"<del>$1</del>").to_string()
    };

    // Spoiler formatting - '||text||' for Markdown or <spoiler>text</spoiler> for HTML
    formatted_text = if format_type == "markdown" {
        let spoiler_re = Regex::new(r"\|\|(.*?)\|\|").unwrap();
        spoiler_re.replace_all(&formatted_text, r"||$1||").to_string()
    } else {
        let spoiler_re = Regex::new(r"\|\|(.*?)\|\|").unwrap();
        spoiler_re.replace_all(&formatted_text, r"<spoiler>$1</spoiler>").to_string()
    };

    formatted_text
}

fn send_message(chat: MessageChat, text: String) -> HttpResponse {
    let mut m = SendMessage::new(chat, text);
    m.parse_mode(ParseMode::Html);
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

fn add_method(value: &mut Value, method: String) {
    match value {
        Value::Object(m) => {
            m.insert("method".to_string(), Value::String(method));
        }
        _ => (),
    }
}
