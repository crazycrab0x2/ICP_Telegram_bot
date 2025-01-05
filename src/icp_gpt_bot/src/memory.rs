use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use crate::types::{Message, MessageType};

type UserDataStore = BTreeMap<String, Message>;
type PromptStore = BTreeMap<String, String>;
type UsernameStore = Vec<String>;

struct Config {
    model: String,
    prompt: String,
    
}  

thread_local! {
    pub static USER_DATA_STORE: RefCell<UserDataStore> = RefCell::default();

    pub static PROMPT_STORE: RefCell<String> = RefCell::default();

    pub static USERNAME_STORE: RefCell<UsernameStore> = RefCell::default();

    pub static ADMIN_STORE: RefCell<String> = RefCell::default();

    pub static TOKEN_STORE: RefCell<String> = RefCell::default();
}

pub fn get_followed_messages(username: String) -> Vec<Message> {
    USER_DATA_STORE.with(|user_data_store| {
        let mut messages: Vec<Message> = user_data_store
            .borrow()
            .iter()
            .filter(|(_, message)| message.username == username)
            .map(|(_, message)| message.clone())
            .collect();
        messages.sort_by(|a, b| a.date.cmp(&b.date));
        let mut followed_message = vec![];
        let mut flag = true;
        messages.iter().for_each(|message| {
            if flag {
                followed_message.push(message.clone());
                flag = message.is_follow;
            } else {
                return;
            }
        });
        followed_message
    })
}

pub fn get_latest_messages(username: String) -> Option<Message> {
    USER_DATA_STORE.with(|user_data_store| {
        let mut messages: Vec<Message> = user_data_store
            .borrow()
            .iter()
            .filter(|(_, message)| message.username == username)
            .map(|(_, message)| message.clone())
            .collect();
        messages.sort_by(|a, b| a.date.cmp(&b.date));
        messages.into_iter().next()
    })
}

pub fn add_new_messages(
    key: String,
    username: String,
    types: MessageType,
    date: u64,
    question: String,
    answer: String,
    is_follow: bool,
) {
    delete_messages(username.clone(), is_follow);
    USER_DATA_STORE.with(|user_data_store| {
        let new_message = Message {
            username,
            date,
            types,
            question,
            answer,
            is_follow,
        };
        user_data_store.borrow_mut().insert(key, new_message);
    });
}

pub fn delete_messages(username: String, is_follow: bool) {
    let time = ic_cdk::api::time();
    let interval: u64 = 30 * 24 * 60 * 60 * 1_000_000_000; // a month in nanosecond
    let mut old_message_keys: Vec<String> = vec![];
    USER_DATA_STORE.with(|user_data_store| {
        let binding = user_data_store.borrow();
        binding
            .iter()
            .filter(|(_, message)| message.username == username && (!is_follow || time - message.date > interval))
            .for_each(|(key, _)| old_message_keys.push(key.clone()))
    });
    USER_DATA_STORE.with(|user_data_store| {
        let mut binding = user_data_store.borrow_mut();
        old_message_keys.iter().for_each(|key| {
            let _ = binding.remove(key);
        })
    });
}

pub fn is_admin(admin: String) -> bool {
    ADMIN_STORE.with(|admin_store| {
        if admin == admin_store.borrow().clone() {
            true
        } else {
            false
        }
    })
}

pub fn is_token_valid(token: String) -> bool {
    TOKEN_STORE.with(|token_store| {
        if token == token_store.borrow().clone() {
            true
        } else {
            false
        }
    })
}

pub fn is_user(username: String) -> bool {
    USERNAME_STORE.with(|username_store| {
        if username_store.borrow().len() == 0 {
            true
        } else {
            if username_store.borrow().contains(&username) {
                true
            } else {
                false
            }
        }
    })
}

pub fn get_prompt() -> String {
    PROMPT_STORE.with(|prompt_store| {
        prompt_store.borrow().clone()
    })
}