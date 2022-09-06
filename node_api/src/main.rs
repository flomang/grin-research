use clap::Parser;
use grin_api::foreign_rpc::foreign_rpc;
use grin_pool::types::PoolEntry;
use grin_util::ToHex;
use log::{debug, info, warn};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
//use std::time::Duration;
use jsonrpc_client::{rpc, rpc_async};
use std::collections::HashSet;
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
    let unconfirmed_inputs = Arc::new(Mutex::new(HashSet::new()));

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

    // needed in first thread
    let unconfirmed_inputs_clone = Arc::clone(&unconfirmed_inputs);
    let grin_url_clone = grin_url.clone();

    // block tip thread
    let handle1 = thread::spawn(move || {
        let mut current_height = 0;
        let delay = time::Duration::from_secs(1);

        loop {
            debug!("block tip thread");

            // retrieve grin block tip
            let result = rpc(&grin_url_clone, &foreign_rpc::get_tip().unwrap());

            match result {
                Ok(Ok(grin_tip)) => {
                    if current_height < grin_tip.height {
                        current_height = grin_tip.height;

                        // read block at height
                        let block = rpc(
                            &grin_url_clone,
                            &foreign_rpc::get_block(Some(current_height), None, None).unwrap(),
                        )
                        .unwrap()
                        .unwrap();

                        let emission = block.header.height * 60 + 60;

                        // does block have inputs?
                        if block.inputs.len() > 0 {
                            info!("new block: {}, supply: {}, inputs: {}", block.header.height, emission, block.inputs.len());

                            block.inputs.iter().for_each(|input| {
                                let mut uncommitted = unconfirmed_inputs_clone.lock().unwrap();
                                if uncommitted.contains(input) {
                                    uncommitted.remove(input);
                                    info!("\tconfirmed input: {}", input);
                                }
                            })
                        } else {
                            info!("empty block: {}, supply: {}", block.header.height, emission);
                        }
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
                        info!("unconfirmed transactions {}", txns.len());

                        all_txns = txns;
                        all_txns.iter().enumerate().for_each(|(i, txn)| {
                            let kernels = txn.tx.body.kernels.len();

                            info!("  trans #{}", i + 1);
                            info!("\tat: {}", txn.tx_at);
                            info!("\tsrc: {:?}", txn.src);
                            info!("\tkernels: {:?}", kernels);

                            let inputs: Vec<String> = match &txn.tx.body.inputs {
                                grin_core::core::transaction::Inputs::FeaturesAndCommit(vec) => {
                                    vec.iter().map(|f| f.commitment().to_hex()).collect()
                                }
                                grin_core::core::transaction::Inputs::CommitOnly(vec) => {
                                    vec.iter().map(|f| f.commitment().to_hex()).collect()
                                }
                            };

                            let outputs = txn.tx.body.outputs.iter().map(|output| {
                                output.identifier.commitment().to_hex()
                            });

                            info!("\tinputs: {:?}", inputs.len());
                            inputs.iter().for_each(|input| {
                                let mut uncommitted = unconfirmed_inputs.lock().unwrap();
                                uncommitted.insert(input.to_owned());

                                info!("\t  unconfirmed: {}", input);
                            });

                            info!("\toutputs: {:?}", outputs.len());
                            outputs.for_each(|output| {
                                info!("\t  unconfirmed: {}", output);
                            });

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
