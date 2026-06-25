#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Node access params - fallback if cookie file isn't present
const RPC_URL: &str = "http://127.0.0.1:18443";
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

fn main() -> bitcoincore_rpc::Result<()> {
    // Attempt connecting using standard cookie authentication first (common in test runners),
    // then fall back to the explicit user/pass combination if that fails.
    let rpc = Client::new(RPC_URL, Auth::None).or_else(|_| {
        Client::new(
            RPC_URL,
            Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
        )
    })?;

    // Get blockchain info to verify connectivity
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?} - main.rs:27", blockchain_info);

    // ==========================================
    // 1. Create/Load Wallets ('Miner' and 'Trader')
    // ==========================================
    println!("Setting up wallets... - main.rs:32");
    let _ = rpc.create_wallet("Miner", Some(false), Some(false), None, None);
    let _ = rpc.create_wallet("Trader", Some(false), Some(false), None, None);

    // Dynamic wallet URI routing to support both auth types cleanly
    let miner_rpc = Client::new("http://127.0.0.1:18443/wallet/Miner", Auth::None).or_else(|_| {
        Client::new(
            "http://127.0.0.1:18443/wallet/Miner",
            Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
        )
    })?;

    let trader_rpc = Client::new("http://127.0.0.1:18443/wallet/Trader", Auth::None).or_else(|_| {
        Client::new(
            "http://127.0.0.1:18443/wallet/Trader",
            Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
        )
    })?;

    // ==========================================
    // 2. Generate spendable balances in Miner wallet
    // ==========================================
    let miner_address = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
    println!("Miner Address: {} - main.rs:55", miner_address);

    /* * EXPLAINER COMMENT: Why it takes 101 blocks to get a spendable wallet balance:
     * According to Bitcoin consensus rules, block rewards (coinbase transactions) 
     * are subject to COINBASE_MATURITY, requiring 100 subsequent confirmations before 
     * they can be spent. Therefore, the reward from the 1st mined block only becomes 
     * an available, spendable balance after mining an additional 100 blocks (101 total).
     */
    println!("Mining blocks until balance is positive... - main.rs:63");
    let mut blocks_mined = 0;
    while miner_rpc.get_balance(None, None)?.to_btc() == 0.0 {
        miner_rpc.generate_to_address(1, &miner_address)?;
        blocks_mined += 1;
    }
    
    let miner_balance = miner_rpc.get_balance(None, None)?;
    println!("Positive balance achieved after {} blocks! - main.rs:71", blocks_mined);

    // ==========================================
    // 3. Load Trader wallet and generate a new address
    // ==========================================
    let trader_address = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();
    println!("Trader Address: {} - main.rs:77", trader_address);

    // ==========================================
    // 4. Send 20 BTC from Miner to Trader
    // ==========================================
    let amount_to_send = Amount::from_btc(20.0).unwrap();
    let txid = miner_rpc.send_to_address(
        &trader_address,
        amount_to_send,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!("Transaction sent! TXID: {} - main.rs:93", txid);

    // ==========================================
    // 5. Check transaction in mempool
    // ==========================================
    let mempool_entry = miner_rpc.get_mempool_entry(&txid)?;

    let tx_info = miner_rpc.get_transaction(&txid, Some(true))?;
    let raw_tx = miner_rpc.get_raw_transaction(&txid, None)?;

    // Identify Miner input details
    let mut miner_input_address = String::from("unknown");
    let mut miner_input_amount_btc = 0.0;

    if !raw_tx.input.is_empty() {
        let prev_out = &raw_tx.input[0].previous_output;
        if let Ok(prev_tx) = miner_rpc.get_transaction(&prev_out.txid, Some(true)) {
            if let Some(detail) = prev_tx.details.iter().find(|d| d.category == bitcoincore_rpc::json::GetTransactionResultDetailCategory::Receive) {
                miner_input_address = detail.address.as_ref().map(|a| a.clone().assume_checked().to_string()).unwrap_or_default();
                miner_input_amount_btc = detail.amount.to_btc();
            }
        }
    }

    // Identify Trader output info and Miner change info
    let mut trader_output_address = trader_address.to_string();
    let trader_output_amount_btc = 20.0;
    let mut miner_change_address = String::from("none");
    let mut miner_change_amount_btc = 0.0;

    for detail in &tx_info.details {
        if detail.category == bitcoincore_rpc::json::GetTransactionResultDetailCategory::Receive {
            miner_change_address = detail.address.as_ref().map(|a| a.clone().assume_checked().to_string()).unwrap_or_default();
            miner_change_amount_btc = detail.amount.to_btc();
        }
    }

    let tx_fees_btc = tx_info.fee.unwrap_or_default().to_btc().abs();

    // ==========================================
    // 6. Mine 1 block to confirm the transaction
    // ==========================================
    println!("Mining 1 block to confirm transaction... - main.rs:135");
    let conf_hashes = miner_rpc.generate_to_address(1, &miner_address)?;
    let block_hash = conf_hashes.first().expect("Failed to mine block");
    
    let block_info = miner_rpc.get_block_info(block_hash)?;
    let block_height = block_info.height;

    // ==========================================
    // 7. Write the data to out.txt
    // ==========================================
    // Check if we are running inside the `rust` subfolder. If so, write out.txt to parent folder `../out.txt`
    let file_path = if Path::new("Cargo.toml").exists() {
        "../out.txt"
    } else {
        "out.txt"
    };

    let mut file = File::create(file_path).expect("Unable to create file");
    
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_input_address)?;
    writeln!(file, "{}", miner_input_amount_btc)?;
    writeln!(file, "{}", trader_output_address)?;
    writeln!(file, "{}", trader_output_amount_btc)?;
    writeln!(file, "{}", miner_change_address)?;
    writeln!(file, "{}", miner_change_amount_btc)?;
    writeln!(file, "-{}", tx_fees_btc)?; 
    writeln!(file, "{}", block_height)?;
    writeln!(file, "{}", block_hash)?;

    println!("All tasks processed successfully! out.txt file written. - main.rs:165");
    Ok(())
}