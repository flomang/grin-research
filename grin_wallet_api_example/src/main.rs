use clap::Parser;
use grin_util::{from_hex, secp_static::static_secp_instance};
use grin_wallet_api::owner_rpc::owner_rpc;
use grin_util::secp::key::{SecretKey, PublicKey};
use grin_wallet_api::ECDHPubkey;
use log::info;
use std::{net::SocketAddr, fs};
use jsonrpc_client::rpc_async_secure;

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

    // use in all tests
	let sec_key_str = "e00dcc4a009e3427c6b1e1a550c538179d46f3827a13ed74c759c860761caf1e";
	//let pub_key_str = "03b3c18c9a38783d105e238953b1638b021ba7456d87a5c085b3bdb75777b4c490";

    let sec_key_bytes = from_hex(sec_key_str).unwrap();
	let pub_key = {
	 	let secp_inst = static_secp_instance();
	 	let secp = secp_inst.lock();
	 	let secret = SecretKey::from_slice(&secp, &sec_key_bytes).unwrap();
        ECDHPubkey{ ecdh_pubkey: PublicKey::from_secret_key(&secp, &secret).unwrap()}
	};

    let dir = format!("{}/.grin/main/.owner_api_secret", home::home_dir().unwrap().display());
    let user = "grin".to_string();
    let pass = fs::read_to_string(dir).unwrap();

    let key = rpc_async_secure(&grin_url, &owner_rpc::init_secure_api(pub_key).unwrap(), user, pass)
        .await
        .unwrap()
        .unwrap();
    

    info!("int secure api key: {:?}", key);

    Ok(())
}
