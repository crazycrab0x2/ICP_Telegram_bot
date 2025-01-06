use crate::gpt::call_chatgpt;
use crate::memory::{get_shortcut_prompt, is_admin, is_valid_user};
use crate::types::{Form, Message, MessageType};
use crate::{
    memory::{
        add_new_messages, add_shortcut_prompt, add_username, get_followed_messages,
        get_latest_messages, get_model, get_prompt, get_shortcuts, get_usernames,
        remove_shortcut_prompt, remove_username, set_model, set_prompt,
    },
    types::{HeaderField, HttpResponse},
};
use serde_json::json;
use serde_json::Value;
use telegram_bot_raw::{MessageChat, ParseMode, SendMessage};

pub async fn handle_message(username: String, chat: MessageChat, text: String) -> HttpResponse {
    let timestamp = ic_cdk::api::time();
    if !is_valid_user(username.clone()) {
        return send_message(chat, "User is not allowed to use this bot.".to_string());
    }
    let response = if text.starts_with("/") {
        if text == "/start".to_string() {
            "Hello! I am a Telegram Bot on Internet Computer using ChatGPT.  \n  \nSend me a question, and I will do my best to answer it. Please be specific, as I'm not very clever.  \nI don't remember chat context by default. To ask follow-up questions, reply to my messages or start your questions with a '+' sign.  \n  \nBuilt-in commands:  \n/retry - retry the last question  \n/imagine - generate described image  \n/config - config for bot  \n/help - show help  \n/info - show info  \n! - use defualt shortcut  \n".to_string()
        } else if text == "/info".to_string() {
            format!(
                "This is a Telegram bot on the Internet Computer!  \nMy canister id: {}  \nLocal time is {}ns.  \nMy cycle balance is {}G Cycle  \nEstimate : {} requests  \nFind me on telegram:  \nhttps://t.me/canister_ai_bot\nFind me on browser:  \nhttps://{}.raw.icp0.io/  \n",
                ic_cdk::id(),
                timestamp,
                (ic_cdk::api::canister_balance() / 1_000_000_000) as i32,
                (ic_cdk::api::canister_balance() / 700_000_000) as i32,
                ic_cdk::id()
            )
        } else if text == "/help".to_string() {
            "/config usernames - get all usernames  \n/config usernames add {username} - add new username  \n/config usernames remove crazycrab0x1 - remove {username}  \n  \n/config model - get current model  \n/config model {model} - set model  \n  \n/config prompt - get current prompt  \n/config prompt {prompt} - set new prompt  \n  \n/config shortcut - get all shortcuts  \n/config shortcut add {shortcut} {prompt} - set new shortcut  \n/config shortcut remove {shortcut} {prompt} - remove shortcut  \n".to_string()
        } else if text == "/retry".to_string() {
            core_action(MessageType::Chat, username, "".to_string(), false, true).await
        } else if text == "/imagine".to_string() {
            "Send prompt after /Imagine\nLike /imagine a cute cat".to_string()
        } else if text.contains("/config") {
            let config_text = text.strip_prefix("/config ");
            if config_text.is_none() {
                format!("Use these subcommand to config bot  \nusernames  \nprompt  \nmodel  \nshortcut  \n")
            } else {
                handle_config(config_text.unwrap(), username.clone())
            }
        } else if text.starts_with("/imagine") {
            let prompt = text.strip_prefix("/imagine ").unwrap();
            core_action(
                MessageType::Image,
                username,
                prompt.to_string(),
                false,
                false,
            )
            .await
        } else {
            "Invalid Command.".to_string()
        }
    } else if text.starts_with("!") {
        let shortcut_text = text.strip_prefix("!");
        if shortcut_text.is_none() {
            format!("Invalid format  \n!shortcut prompt")
        } else {
            let mut words = shortcut_text.unwrap().split_whitespace();
            let shortcut = words.next();
            if shortcut.is_none() || words.next().is_none() {
                format!("Invalid format  \n!shortcut prompt")
            } else {
                let shortcut_prompt = get_shortcut_prompt(shortcut.unwrap().to_string());
                if shortcut_prompt.is_none() {
                    format!("Invalid shortcut.")
                } else {
                    let user_prompt = shortcut_text
                        .unwrap()
                        .strip_prefix(format!("{} ", shortcut.unwrap()).as_str())
                        .unwrap();
                    let prompt = shortcut_prompt.unwrap() + user_prompt;
                    core_action(MessageType::Chat, username, prompt, false, false).await
                }
            }
        }
    } else {
        let is_follow = if text.starts_with('+') { true } else { false };
        core_action(MessageType::Chat, username, text, is_follow, false).await
    };
    send_message(chat, response)
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
        reply = call_chatgpt(uri, request_body, key.clone()).await;
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
    reply
}

fn handle_config(config_text: &str, username: String) -> String {
    let is_admin = is_admin(username);

    if config_text.starts_with("usernames") {
        let username_config_text = config_text.strip_prefix("usernames ");
        if username_config_text.is_none() {
            let usernames = get_usernames();
            if usernames.len() == 0 {
                format!("There is no specific user.")
            } else {
                usernames.concat()
            }
        } else {
            let mut words = username_config_text.unwrap().split_whitespace();
            let sub_command = words.next().unwrap();
            let new_username = words.next();
            if new_username.is_none() {
                format!("Invalid format.  \n/config usernames add/remove **username**")
            } else {
                if sub_command == "add" {
                    if !is_admin {
                        return format!("You are not admin.");
                    }
                    add_username(new_username.unwrap().to_string());
                    format!("username **{}** added successfully.", new_username.unwrap())
                } else if sub_command == "remove" {
                    if !is_admin {
                        return format!("You are not admin.");
                    }
                    remove_username(new_username.unwrap().to_string());
                    format!(
                        "username **{}** removed successfully.",
                        new_username.unwrap()
                    )
                } else {
                    format!("Invalid subcommand.  \nYou can use **add** and **remove**")
                }
            }
        }
    } else if config_text.starts_with("model") {
        let model_config_text = config_text.strip_prefix("model ");
        if model_config_text.is_none() {
            get_model()
        } else {
            if is_admin {
                set_model(model_config_text.unwrap().to_string());
                format!("Model set by '{}'", model_config_text.unwrap())
            } else {
                format!("You are not admin.")
            }
        }
    } else if config_text.starts_with("prompt") {
        let prompt_config_text = config_text.strip_prefix("prompt ");
        if prompt_config_text.is_none() {
            get_prompt()
        } else {
            if is_admin {
                set_prompt(prompt_config_text.unwrap().to_string());
                format!("Prompt set for '{}'", prompt_config_text.unwrap())
            } else {
                format!("You are not admin.")
            }
        }
    } else if config_text.starts_with("shortcut") {
        let shortcut_config_text = config_text.strip_prefix("shortcut ");
        if shortcut_config_text.is_none() {
            let shortcuts = get_shortcuts();
            if shortcuts.len() == 0 {
                format!("There is no any shortcut.")
            } else {
                shortcuts.concat()
            }
        } else {
            let sub_command = shortcut_config_text
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap();
            if sub_command == "add" {
                if !is_admin {
                    return format!("You are not admin.");
                }
                let new_shortcut = shortcut_config_text.unwrap().strip_prefix("add ");
                if new_shortcut.is_none() {
                    format!("Invalid format.  \n/config shortcut add **shortcut** **prompt**")
                } else {
                    let mut words = new_shortcut.unwrap().split_whitespace();
                    let shortcut = words.next();
                    if shortcut.is_none() || words.next().is_none() {
                        format!("Invalid format.  \n/config shortcut add **shortcut** **prompt**")
                    } else {
                        let prompt = new_shortcut
                            .unwrap()
                            .strip_prefix(format!("{} ", shortcut.unwrap()).as_str())
                            .unwrap();
                        add_shortcut_prompt(shortcut.unwrap().to_string(), prompt.to_string());
                        format!("Shortcut **{}** added successfully.", shortcut.unwrap())
                    }
                }
            } else if sub_command == "remove" {
                if !is_admin {
                    return format!("You are not admin.");
                }
                let shortcut = shortcut_config_text.unwrap().strip_prefix("remove ");
                if shortcut.is_none() {
                    format!("Invalid format.  \n/config shortcut remove **shortcut**")
                } else {
                    if remove_shortcut_prompt(shortcut.unwrap().to_string()).is_none() {
                        format!("Shortcut **{}** doesn't exist", shortcut.unwrap())
                    } else {
                        format!("Shortcut **{}** removed successfully.", shortcut.unwrap())
                    }
                }
            } else {
                let shortcut_prompt = get_shortcut_prompt(sub_command.to_string());
                if shortcut_prompt.is_none() {
                    format!("Shortcut did not add for '{}'", sub_command)
                } else {
                    shortcut_prompt.unwrap()
                }
            }
        }
    } else {
        "Invalid config command.".to_string()
    }
}

fn make_chat_request(old_messages: Vec<Message>, is_retry: bool, prompt: String) -> String {
    let system_prompt = get_prompt();
    let model = get_model();
    let mut messages = vec![
        Form {
            role: "system".to_string(),
            content: system_prompt,
        },
        // Form {
        //     role: "developer".to_string(),
        //     content: "Please give me response as telegram Markdown parse format.".to_string(),
        // },
    ];

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
        "model": model,
        "messages": messages
    })
    .to_string()
}

fn send_message(chat: MessageChat, text: String) -> HttpResponse {
    let mut m = SendMessage::new(chat, text);
    m.parse_mode(ParseMode::Markdown);
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
