//use clap::Parser;
use easy_jsonrpc_mw::{BoundMethod, Response};
//use grin_api::foreign_rpc::foreign_rpc;
//use grin_pool::types::PoolEntry;
//use log::info;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
//use std::{thread, time};


// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
// struct Args {
//    /// supported grin api version 
//    #[clap(long, env)]
//    grin_api_version: String,

//    /// supported grin api version 
//    #[clap(long, env)]
//    grin_api_addr: String,
// }


// Demonstrate an example JSON-RCP call against grin.
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // init env from .env file
//     dotenv::dotenv().ok();
//     // color logs
//     pretty_env_logger::init();

//     let args = Args::parse();
//     info!("{:?}", args);

//     // this is the 
//     let grin_addr: SocketAddr = args.grin_api_addr
//         .parse()
//         .unwrap();

//     let grin_version = rpc(&grin_addr, &foreign_rpc::get_version().unwrap())
//         .await??
//         .node_version;

//     if args.grin_api_version != grin_version {
//         panic!(
//             "expected grin version: {} actual running node instance is: {}",
//             args.grin_api_version, grin_version
//         )
//     }

//     info!("grin api: {:?}", grin_version);

//     let delay = time::Duration::from_secs(1);
//     let mut all_txns: Vec<PoolEntry> = vec![];

//     let grin_tip = rpc(&grin_addr, &foreign_rpc::get_tip().unwrap()).await??;
//     let mut current_height = grin_tip.height;
//     info!("height: {:?}", current_height);

//     while let Ok(txns) = rpc(&grin_addr, &foreign_rpc::get_unconfirmed_transactions().unwrap()).await? {
//         let grin_tip = rpc(&grin_addr, &foreign_rpc::get_tip().unwrap()).await??;

//         if current_height < grin_tip.height {
//             current_height = grin_tip.height;
//             let block = rpc(&grin_addr, &foreign_rpc::get_block(Some(current_height), None, None).unwrap()).await??;
//             info!("Supply: {}", block.header.height * 60 + 60);
//             info!("new block: {}\n{:#?}", block.header.height, block);
//         }

//         if all_txns.len() != txns.len() {
//             all_txns = txns;
//             for txn in all_txns.iter() {
//                 let inputs = txn.tx.body.inputs.len();
//                 let outputs = txn.tx.body.outputs.len();
//                 let kernels = txn.tx.body.kernels.len();

//                 info!("----");
//                 info!("\t at: {}", txn.tx_at);
//                 info!("\t src: {:?}", txn.src);
//                 info!("\t kernels: {:?}", kernels);
//                 info!("\t inputs: {:?}", inputs);
//                 info!("\t outputs: {:?}", outputs);
//                 info!("\t tx: {:?}", txn.tx);
//             }
//         }
//         thread::sleep(delay);
//     }

//     Ok(())
// }

pub async fn rpc<R: Deserialize<'static>>(
    addr: &SocketAddr,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post(addr, &request.as_request()).await?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

async fn post(addr: &SocketAddr, body: &Value) -> Result<Value, reqwest::Error> {
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
