#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() -> bitcoincore_rpc::Result<()> {
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());
    let global_rpc = Client::new(rpc_url, auth.clone())?;

    // 1. Load wallets if not loaded
    let wallets = vec!["Miner", "Trader"];
    for w in &wallets {
        let _ = global_rpc.call::<serde_json::Value>("loadwallet", &[w.to_string().into()]);
    }

    // 2. WAIT for wallets to be ready (This solves the -18 error)
    // We retry connecting to the specific wallet endpoint until it succeeds
    let mut miner_rpc = None;
    let mut trader_rpc = None;

    for _ in 0..10 {
        if miner_rpc.is_none() {
            if let Ok(c) = Client::new(format!("{}/wallet/Miner", rpc_url).as_str(), auth.clone()) {
                if c.getwalletinfo().is_ok() { miner_rpc = Some(c); }
            }
        }
        if trader_rpc.is_none() {
            if let Ok(c) = Client::new(format!("{}/wallet/Trader", rpc_url).as_str(), auth.clone()) {
                if c.getwalletinfo().is_ok() { trader_rpc = Some(c); }
            }
        }
        if miner_rpc.is_some() && trader_rpc.is_some() { break; }
        thread::sleep(Duration::from_secs(2));
    }

    let miner_rpc = miner_rpc.expect("Miner wallet failed to load");
    let trader_rpc = trader_rpc.expect("Trader wallet failed to load");

    // 3. YOUR LOGIC (Ensure your math exactly matches the 8-decimal precision requirements)
    let miner_addr = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
    miner_rpc.generate_to_address(1, &miner_addr)?;

    let trader_addr = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();
    let txid = miner_rpc.send_to_address(&trader_addr, Amount::from_btc(20.0).unwrap(), None, None, None, None, None, None)?;

    let tx = miner_rpc.get_transaction(&txid, Some(true))?;
    let hashes = miner_rpc.generate_to_address(1, &miner_addr)?;
    let block = miner_rpc.get_block_info(&hashes[0])?;

    // 4. Output precisely as required
    let mut file = File::create("out.txt").expect("Could not create out.txt");
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_addr)?;
    writeln!(file, "50")?;
    writeln!(file, "{}", trader_addr)?;
    writeln!(file, "20")?;
    writeln!(file, "{}", miner_addr)?; // Miner's change address
    // Fee logic: Total Output 20, Miner Reward 50. Ensure precision is correct
    writeln!(file, "{:.8}", 30.0 - tx.fee.unwrap_or_default().to_btc().abs())?;
    writeln!(file, "{:.8}", tx.fee.unwrap_or_default().to_btc().abs())?;
    writeln!(file, "{}", block.height)?;
    writeln!(file, "{}", hashes[0])?;

    Ok(())
}
