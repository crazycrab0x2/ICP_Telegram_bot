use crate::types::{Message, MessageType};
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::types::Config;

type UserDataStore = BTreeMap<String, Message>;
type UsernameStore = Vec<String>;
type ShortcutStore = BTreeMap<String, String>;

thread_local! {
    pub static USER_DATA_STORE: RefCell<UserDataStore> = RefCell::default();

    pub static USERNAME_STORE: RefCell<UsernameStore> = RefCell::default();

    pub static SHORTCUT_STORE: RefCell<ShortcutStore> = RefCell::default();

    pub static CONFIG_STORE: RefCell<Config> = RefCell::new(Config::default());
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
            .filter(|(_, message)| {
                message.username == username && (!is_follow || time - message.date > interval)
            })
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
    CONFIG_STORE.with(|config_store| {
        if admin == config_store.borrow().clone().admin {
            true
        } else {
            false
        }
    })
}

pub fn is_token_valid(token: String) -> bool {
    CONFIG_STORE.with(|config_store| {
        if token == config_store.borrow().clone().token {
            true
        } else {
            false
        }
    })
}

pub fn is_valid_user(username: String) -> bool {
    is_admin(username.clone())
        || USERNAME_STORE.with(|username_store| {
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

pub fn get_model() -> String {
    CONFIG_STORE.with(|config_store| config_store.borrow().clone().model)
}

pub fn set_model(model: String) {
    CONFIG_STORE.with(|config_store| {
        let mut config = config_store.borrow().clone();
        config.model = model;
        *config_store.borrow_mut() = config;
    })
}

pub fn get_prompt() -> String {
    CONFIG_STORE.with(|config_store| config_store.borrow().clone().prompt)
}

pub fn set_prompt(prompt: String) {
    CONFIG_STORE.with(|config_store| {
        let mut config = config_store.borrow().clone();
        config.prompt = prompt;
        *config_store.borrow_mut() = config;
    })
}

pub fn get_shortcuts() -> Vec<String> {
    SHORTCUT_STORE.with(|shortcut_store| shortcut_store.borrow().iter().map(|(shortcut, _)| {
        format!("{}\n",shortcut.clone())
    }).collect())
}

pub fn get_shortcut_prompt(shortcut: String) -> Option<String> {
    SHORTCUT_STORE.with(|shortcut_store| shortcut_store.borrow().get(&shortcut).cloned())
}

pub fn add_shortcut_prompt(shortcut: String, prompt: String) {
    SHORTCUT_STORE.with(|shortcut_store| shortcut_store.borrow_mut().insert(shortcut, prompt));
}

pub fn remove_shortcut_prompt(shortcut: String) -> Option<String> {
    SHORTCUT_STORE.with(|shortcut_store| shortcut_store.borrow_mut().remove(&shortcut))
}

pub fn get_usernames() -> Vec<String> {
    USERNAME_STORE.with(|username_store| {
        username_store.borrow().iter().map(|username| format!("{}  \n", username.clone())).collect()
    })
}

pub fn add_username(username: String) {
    USERNAME_STORE.with(|username_store| {
        if !username_store.borrow().contains(&username){
            username_store.borrow_mut().push(username)
        }
    })
}

pub fn remove_username(username: String) -> Option<String> {
    USERNAME_STORE.with(|username_store| {
        let mut binding = username_store.borrow_mut();
        if binding.contains(&username) {
            binding.retain_mut(|user| *user != username);
            Some(username)
        } else {
            None
        }
    })
}