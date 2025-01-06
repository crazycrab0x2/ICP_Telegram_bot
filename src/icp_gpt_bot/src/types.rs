use candid::CandidType;
use serde::{Deserialize, Serialize};

/// A key-value pair for a HTTP header.
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
#[derive(PartialEq)]
pub struct HeaderField(pub String, pub String);

/// The important components of an HTTP request.
#[derive(CandidType, Deserialize, Clone)]
pub struct HttpRequest {
    /// The HTTP method string.
    pub method: String,
    /// The URL that was visited.
    pub url: String,
    /// The request headers.
    pub headers: Vec<HeaderField>,
    /// The request body.
    // #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

/// A HTTP response.
#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status_code: u16,
    /// The response header map.
    pub headers: Vec<HeaderField>,
    /// The response body.
    // #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    /// Whether the query call should be upgraded to an update call.
    pub upgrade: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct TransformArgs {
    /// Raw response from remote service, to be transformed
    pub response: HttpResponse,

    /// Context for response transformation
    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

#[derive(Clone, Serialize, CandidType, Deserialize, PartialEq, Debug)]
pub enum MessageType {
    Chat,
    Image
}


#[derive(Clone, Serialize, CandidType, Deserialize)]
pub struct Message {
    pub username: String,
    pub date: u64,
    pub types: MessageType,
    pub question: String,
    pub answer: String,
    pub is_follow: bool
}

#[derive(Clone, Serialize, CandidType, Deserialize)]
pub struct Form {
    pub role: String,
    pub content: String
}

#[derive(Clone, Serialize, CandidType, Deserialize)]
pub struct Config {
    pub admin: String,
    pub token: String,
    pub model: String,
    pub prompt: String,
}  

impl Default for Config {
    fn default() -> Self {
        Self {
            admin: String::from(""),
            token: String::from(""),
            model: String::from(""),
            prompt: String::from(""),
        }
    }
}