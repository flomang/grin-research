use easy_jsonrpc_mw::{BoundMethod, Response};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

pub fn rpc<R: Deserialize<'static>>(
    url: &str,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post(url, &request.as_request())?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

pub async fn rpc_async<R: Deserialize<'static>>(
    url: &str,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post_async(url, &request.as_request()).await?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

fn post(url: &str, body: &Value) -> Result<Value, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(url)
        .json(body)
        .send()?
        .error_for_status()?
        .json()
}

async fn post_async(url: &str, body: &Value) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let response = client
        .post(url)
        .json(body)
        .send()
        .await?;

    let json_response = response.error_for_status()?.json::<Value>().await?;

    Ok(json_response)
}


#[derive(Debug)]
pub enum RpcErr {
    Http(reqwest::Error),
    InvalidResponse,
}

impl From<easy_jsonrpc_mw::InvalidResponse> for RpcErr {
    fn from(_other: easy_jsonrpc_mw::InvalidResponse) -> Self {
        RpcErr::InvalidResponse
    }
}

impl From<easy_jsonrpc_mw::ResponseFail> for RpcErr {
    fn from(_other: easy_jsonrpc_mw::ResponseFail) -> Self {
        RpcErr::InvalidResponse
    }
}

impl From<reqwest::Error> for RpcErr {
    fn from(other: reqwest::Error) -> Self {
        RpcErr::Http(other)
    }
}

impl std::fmt::Display for RpcErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcErr::Http(e) => write!(f, "rpc encountered some http error: {}", e),
            _ => write!(f, "InvalidResponse"),
        }
    }
}

impl std::error::Error for RpcErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RpcErr::Http(e) => Some(e),
            _ => Some(self),
        }
    }
}