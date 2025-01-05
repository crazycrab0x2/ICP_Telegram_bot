use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
    TransformContext as TransformContextCdk,
};
use serde_json::json;


pub async fn call_chatgpt(uri: &str, reqeust_body: String, key: String) -> String {
    let url = format!("https://us-central1-telegram-gpt-488cd.cloudfunctions.net/chatgpt/{}", uri);
    
    let body = json!({
        "request": reqeust_body,
        "key": key
    }).to_string();

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(50_000), // 3MB
        transform: Some(TransformContextCdk::from_name(
            "transform".to_string(),
            vec![],
        )),
    };

    let cycles = 700_000_000;

    match http_request(request, cycles).await {
        Ok((response,)) => String::from_utf8(response.body)
            .unwrap_or_else(|_| "Failed to parse response".to_string()),
        Err((r, m)) => {
            format!("HTTP request failed with code {:?}: {}", r, m)
        }
    }
}

