use clap::Parser;
use grin_api::foreign_rpc::foreign_rpc;
//use grin_pool::types::PoolEntry;
use log::info;
use std::net::SocketAddr;
use std::sync::mpsc;
//use std::time::Duration;
use std::{thread, time};
use api::rpc;

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

        let mut current_height = 0;
        let delay = time::Duration::from_secs(1);
        //let mut all_txns: Vec<PoolEntry> = vec![];

        loop {
            let grin_tip = rpc(&grin_addr, &foreign_rpc::get_tip().unwrap())
                .unwrap()
                .unwrap();

            if current_height < grin_tip.height {
                current_height = grin_tip.height;

                let block = rpc(
                    &grin_addr,
                    &foreign_rpc::get_block(Some(current_height), None, None).unwrap(),
                )
                .unwrap()
                .unwrap();

                info!("supply: {}", block.header.height * 60 + 60);
                info!("block tip at: {}", block.header.height);
            }

            // if let Ok(txns) = rpc(
            //     &grin_addr,
            //     &foreign_rpc::get_unconfirmed_transactions().unwrap(),
            // )
            // .unwrap()
            // {

            //     if all_txns.len() != txns.len() {
            //         all_txns = txns;
            //         for txn in all_txns.iter() {
            //             let inputs = txn.tx.body.inputs.len();
            //             let outputs = txn.tx.body.outputs.len();
            //             let kernels = txn.tx.body.kernels.len();

            //             info!("----");
            //             info!("\t at: {}", txn.tx_at);
            //             info!("\t src: {:?}", txn.src);
            //             info!("\t kernels: {:?}", kernels);
            //             info!("\t inputs: {:?}", inputs);
            //             info!("\t outputs: {:?}", outputs);
            //             info!("\t tx: {:?}", txn.tx);
            //         }
            //     }
            // }
            thread::sleep(delay);
        }
    });

    for received in rx {
        println!("Got: {}", received);
    }

    Ok(())
}
