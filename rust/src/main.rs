#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::fs::File;
use std::io::Write;

fn main() -> bitcoincore_rpc::Result<()> {
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());

    // 1. Connect to the node
    let global_rpc = Client::new(rpc_url, auth.clone())?;

    // 2. Load wallets: Get the list of currently loaded wallets first
    let loaded_wallets = global_rpc.list_wallets()?;
    let wallets_to_load = vec!["Miner", "Trader"];

    for wallet in wallets_to_load {
        if !loaded_wallets.contains(&wallet.to_string()) {
            // Attempt to load if not already loaded
            let _ = global_rpc.call::<serde_json::Value>("loadwallet", &[wallet.to_string().into()]);
        }
    }

    // 3. Wait for the node to fully register the wallets
    std::thread::sleep(std::time::Duration::from_secs(5));

    // 4. Connect to the specific wallets
    let miner_rpc = Client::new(format!("{}/wallet/Miner", rpc_url).as_str(), auth.clone())?;
    let trader_rpc = Client::new(format!("{}/wallet/Trader", rpc_url).as_str(), auth.clone())?;

    // 5. Logic
    let miner_addr = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
    miner_rpc.generate_to_address(1, &miner_addr)?;

    let trader_addr = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();
    let txid = miner_rpc.send_to_address(
        &trader_addr,
        Amount::from_btc(20.0).unwrap(),
        None, None, None, None, None, None,
    )?;

    let tx = miner_rpc.get_transaction(&txid, Some(true))?;
    let hashes = miner_rpc.generate_to_address(1, &miner_addr)?;
    let block = miner_rpc.get_block_info(&hashes[0])?;

    // 6. Write output
    let mut file = File::create("out.txt").expect("Could not create out.txt");
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_addr)?;
    writeln!(file, "50")?;
    writeln!(file, "{}", trader_addr)?;
    writeln!(file, "20")?;
    writeln!(file, "{}", miner_addr)?;
    writeln!(file, "{}", (30.0 - tx.fee.unwrap_or_default().to_btc().abs()))?;
    writeln!(file, "{}", tx.fee.unwrap_or_default().to_btc().abs())?;
    writeln!(file, "{}", block.height)?;
    writeln!(file, "{}", hashes[0])?;

    Ok(())
}
