use clap::Parser;
use easy_jsonrpc_mw::{BoundMethod, Response};
use futures::executor::block_on;
use grin_api::foreign_rpc::foreign_rpc;
use grin_pool::types::PoolEntry;
use log::info;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::Duration;
use std::{thread, time};

#[derive(Clone, Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// supported grin api version
    #[clap(long, env)]
    grin_api_version: String,

    /// supported grin api version
    #[clap(long, env)]
    grin_api_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init env from .env file
    dotenv::dotenv().ok();
    // color logs
    pretty_env_logger::init();

    let args = Args::parse();
    let grin_addr: SocketAddr = args.grin_api_addr.parse().unwrap();

    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    let args1 = args.clone();

    thread::spawn(move || {
        let grin_version = rpc(&grin_addr, &foreign_rpc::get_version().unwrap())
            .unwrap()
            .unwrap()
            .node_version;

        if args1.grin_api_version != grin_version {
            panic!(
                "expected grin version: {} actual running node instance is: {}",
                args1.grin_api_version, grin_version
            )
        }

        info!("grin api: {:?}", grin_version);
        tx1.send(grin_version).unwrap();

        let delay = time::Duration::from_secs(1);
        let mut all_txns: Vec<PoolEntry> = vec![];

        let grin_tip = rpc(&grin_addr, &foreign_rpc::get_tip().unwrap())
            .unwrap()
            .unwrap();
        let mut current_height = grin_tip.height;
        info!("height: {:?}", current_height);

        loop {
            if let Ok(txns) = rpc(
                &grin_addr,
                &foreign_rpc::get_unconfirmed_transactions().unwrap(),
            )
            .unwrap()
            {
                let grin_tip = rpc(&grin_addr, &foreign_rpc::get_tip().unwrap()).unwrap().unwrap();

                if current_height < grin_tip.height {
                    current_height = grin_tip.height;
                    let block = rpc(
                        &grin_addr,
                        &foreign_rpc::get_block(Some(current_height), None, None).unwrap(),
                    )
                    .unwrap().unwrap();
                    info!("Supply: {}", block.header.height * 60 + 60);
                    info!("new block: {}\n{:#?}", block.header.height, block);
                }

                if all_txns.len() != txns.len() {
                    all_txns = txns;
                    for txn in all_txns.iter() {
                        let inputs = txn.tx.body.inputs.len();
                        let outputs = txn.tx.body.outputs.len();
                        let kernels = txn.tx.body.kernels.len();

                        info!("----");
                        info!("\t at: {}", txn.tx_at);
                        info!("\t src: {:?}", txn.src);
                        info!("\t kernels: {:?}", kernels);
                        info!("\t inputs: {:?}", inputs);
                        info!("\t outputs: {:?}", outputs);
                        info!("\t tx: {:?}", txn.tx);
                    }
                }
            }
            thread::sleep(delay);
        }
    });

    for received in rx {
        println!("Got: {}", received);
    }

    Ok(())
}

fn rpc<R: Deserialize<'static>>(
    addr: &SocketAddr,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post(addr, &request.as_request())?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

fn post(addr: &SocketAddr, body: &Value) -> Result<Value, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(&format!("http://{}/v2/foreign", addr))
        .json(body)
        .send()?
        .error_for_status()?
        .json()
}

async fn rpc_async<R: Deserialize<'static>>(
    addr: &SocketAddr,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post_async(addr, &request.as_request()).await?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

async fn post_async(addr: &SocketAddr, body: &Value) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let response = client
        .post(&format!("http://{}/v2/foreign", addr))
        .json(body)
        .send()
        .await?;

    let json_response = response.error_for_status()?.json::<Value>().await?;

    Ok(json_response)
}

use std::fmt;

#[derive(Debug)]
enum RpcErr {
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

impl fmt::Display for RpcErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
