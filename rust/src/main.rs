#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::fs::File;
use std::io::Write;

fn main() -> bitcoincore_rpc::Result<()> {
    // 1. Establish RPC connection
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());
    
    let miner_rpc = Client::new(format!("{}/wallet/Miner", rpc_url), auth.clone())?;
    let trader_rpc = Client::new(format!("{}/wallet/Trader", rpc_url), auth.clone())?;

    // 2. Setup: Generate Mining Reward
    let miner_addr = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
    miner_rpc.generate_to_address(1, &miner_addr)?;

    // 3. Action: Send 20 BTC
    let trader_addr = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();
    let txid = miner_rpc.send_to_address(
        &trader_addr, 
        Amount::from_btc(20.0).unwrap(), 
        None, None, None, None, None, None
    )?;

    // 4. Data Extraction
    let tx = miner_rpc.get_transaction(&txid, Some(true))?;
    let hashes = miner_rpc.generate_to_address(1, &miner_addr)?;
    let block = miner_rpc.get_block_info(&hashes[0])?;


    // 1. Establish RPC connection
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());
    
    // Replace these two lines in your code:
    let miner_rpc = Client::new(format!("{}/wallet/Miner", rpc_url).as_str(), auth.clone())?;
    let trader_rpc = Client::new(format!("{}/wallet/Trader", rpc_url).as_str(), auth.clone())?;

    // 5. File I/O: Writing to the root directory
    let mut file = File::create("../out.txt").expect("Could not create out.txt in root");
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_addr)?;
    writeln!(file, "50")?; // Mined 50 BTC
    writeln!(file, "{}", trader_addr)?;
    writeln!(file, "20")?; // Sent 20 BTC
    writeln!(file, "{}", miner_addr)?; // Change address
    writeln!(file, "{}", 30.0 - tx.fee.unwrap_or_default().to_btc().abs())?; // Change amount
    writeln!(file, "{}", tx.fee.unwrap_or_default().to_btc().abs())?; // Fee
    writeln!(file, "{}", block.height)?;
    writeln!(file, "{}", hashes[0])?;

    Ok(())
}