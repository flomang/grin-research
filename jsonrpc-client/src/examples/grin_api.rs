use clap::Parser;
use grin_api::foreign_rpc::foreign_rpc;
use grin_pool::types::PoolEntry;
use log::{debug, info, warn};
use std::net::SocketAddr;
//use std::sync::mpsc;
//use std::time::Duration;
use api::{rpc, rpc_async};
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
    let grin_url = format!("http://{}/v2/foreign", grin_addr);

    //let (tx, rx) = mpsc::channel();
    //let tx1 = tx.clone();
    //let args1 = args.clone();

    let grin_version = rpc_async(&grin_url, &foreign_rpc::get_version().unwrap())
        .await
        .unwrap()
        .unwrap()
        .node_version;

    assert!(
        args.grin_api_version == grin_version,
        "unexpected grin node version"
    );
    info!("grin api: {:?}", grin_version);

    let grin_url_copy = grin_url.clone();
    // block tip thread
    let handle1 = thread::spawn(move || {
        let mut current_height = 0;
        let delay = time::Duration::from_secs(1);

        loop {
            debug!("block tip thread");

            let result = rpc(&grin_url_copy, &foreign_rpc::get_tip().unwrap());

            match result {
                Ok(Ok(grin_tip)) => {
                    if current_height < grin_tip.height {
                        current_height = grin_tip.height;

                        let block = rpc(
                            &grin_url_copy,
                            &foreign_rpc::get_block(Some(current_height), None, None).unwrap(),
                        )
                        .unwrap()
                        .unwrap();

                        info!("supply: {}", block.header.height * 60 + 60);
                        info!("height: {}", block.header.height);
                    }
                }
                Ok(Err(err)) => {
                    warn!("encountered error: {}", err);
                }
                Err(err) => {
                    info!("encountered rpc error: {}", err);
                    break;
                }
            }

            thread::sleep(delay);
        }
    });

    let handle2 = thread::spawn(move || {
        let delay = time::Duration::from_millis(300);
        let mut all_txns: Vec<PoolEntry> = vec![];

        loop {
            debug!("unconfirmed txns thread");

            let result = rpc(
                &grin_url,
                &foreign_rpc::get_unconfirmed_transactions().unwrap(),
            );

            match result {
                Ok(Ok(txns)) => {
                    if all_txns.len() != txns.len() {
                        info!("Unconfirmed transactions ({})", txns.len());

                        all_txns = txns;
                        all_txns.iter().enumerate().for_each(|(i, txn)| {
                            let inputs = txn.tx.body.inputs.len();
                            let outputs = txn.tx.body.outputs.len();
                            let kernels = txn.tx.body.kernels.len();

                            info!("\ttrans #{}:", i+1);
                            info!("\tat: {}", txn.tx_at);
                            info!("\tsrc: {:?}", txn.src);
                            info!("\tkernels: {:?}", kernels);
                            info!("\tinputs: {:?}", inputs);
                            info!("\toutputs: {:?}", outputs);
                            //info!("\ttx: {:#?}", txn.tx);
                        });
                    }
                }
                Ok(Err(err)) => {
                    warn!("encountered error: {}", err);
                }
                Err(err) => {
                    info!("encountered rpc error: {}", err);
                    break;
                }
            }

            thread::sleep(delay);
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    Ok(())
}
