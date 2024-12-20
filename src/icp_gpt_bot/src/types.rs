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
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
#[derive(PartialEq)]
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

#[derive(CandidType, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformArgs {
    /// Raw response from remote service, to be transformed
    pub response: HttpResponse,

    /// Context for response transformation
    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}
