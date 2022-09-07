use clap::Parser;
use grin_wallet_api::foreign_rpc::foreign_rpc;
use log::info;
use std::net::SocketAddr;
use jsonrpc_client::{rpc, rpc_async};

#[derive(Clone, Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// supported grin api version
    #[clap(long, env)]
    grin_api_version: String,

    /// supported grin api version
    #[clap(long, env)]
    grin_api_wallet_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init env from .env file
    dotenv::dotenv().ok();
    // color logs
    pretty_env_logger::init();

    let args = Args::parse();
    let grin_addr: SocketAddr = args.grin_api_wallet_addr.parse().unwrap();
    let grin_url = format!("http://{}/v3/owner", grin_addr);

    let version = rpc_async(&grin_url, &foreign_rpc::check_version().unwrap())
        .await
        .unwrap()
        .unwrap();
    

    info!("grin api: {:?}", version);
    

    Ok(())
}
