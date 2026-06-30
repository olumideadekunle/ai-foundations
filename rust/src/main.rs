#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> bitcoincore_rpc::Result<()> {
    // 1. Setup connection using cookie authentication
    let cookie_path = Path::new("regtest/data/.cookie");
    let fallback_path = Path::new("../regtest/data/.cookie");

    let auth = if cookie_path.exists() {
        Auth::CookieFile(cookie_path.to_path_buf())
    } else if fallback_path.exists() {
        Auth::CookieFile(fallback_path.to_path_buf())
    } else {
        Auth::None
    };

    let rpc = Client::new("http://127.0.0.1:18443", auth.clone())?;

    // 2. Setup wallets
    let _ = rpc.create_wallet("Miner", Some(false), Some(false), None, None);
    let _ = rpc.create_wallet("Trader", Some(false), Some(false), None, None);

    let miner_rpc = Client::new("http://127.0.0.1:18443/wallet/Miner", auth.clone())?;
    let trader_rpc = Client::new("http://127.0.0.1:18443/wallet/Trader", auth.clone())?;

    // 3. Generate addresses and mine until balance exists
    let miner_address = miner_rpc
        .get_new_address(Some("Mining Reward"), None)?
        .assume_checked();

    /* * EXPLAINER COMMENT: Why it takes 101 blocks to get a spendable wallet balance:
     * According to Bitcoin consensus rules, block rewards (coinbase transactions)
     * are subject to COINBASE_MATURITY, requiring 100 subsequent confirmations before
     * they can be spent. Therefore, the reward from the 1st mined block only becomes
     * an available, spendable balance after mining an additional 100 blocks (101 total).
     */
    while miner_rpc.get_balance(None, None)?.to_btc() == 0.0 {
        miner_rpc.generate_to_address(1, &miner_address)?;
    }

    // 4. Send 20 BTC to Trader
    let trader_address = trader_rpc
        .get_new_address(Some("Received"), None)?
        .assume_checked();
    let txid = miner_rpc.send_to_address(
        &trader_address,
        Amount::from_btc(20.0).unwrap(),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    // 5. Extract transaction details
    let tx_info = miner_rpc.get_transaction(&txid, Some(true))?;
    let raw_tx = miner_rpc.get_raw_transaction(&txid, None)?;

    let mut miner_input_address = miner_address.to_string();
    let mut miner_input_amount_btc = 50.0;

    if !raw_tx.input.is_empty() {
        let prev_out = &raw_tx.input[0].previous_output;
        if let Ok(prev_tx) = miner_rpc.get_transaction(&prev_out.txid, Some(true)) {
            for detail in &prev_tx.details {
                if detail.category
                    == bitcoincore_rpc::json::GetTransactionResultDetailCategory::Receive
                {
                    if let Some(ref addr) = detail.address {
                        miner_input_address = addr.clone().assume_checked().to_string();
                        miner_input_amount_btc = detail.amount.to_btc();
                    }
                }
            }
        }
    }

    let trader_output_address = trader_address.to_string();
    let trader_output_amount_btc = 20.0;

    let mut miner_change_address = miner_address.to_string();
    let mut miner_change_amount_btc = miner_input_amount_btc
        - trader_output_amount_btc
        - tx_info.fee.unwrap_or_default().to_btc().abs();

    for detail in &tx_info.details {
        if detail.category == bitcoincore_rpc::json::GetTransactionResultDetailCategory::Receive {
            if let Some(ref addr) = detail.address {
                miner_change_address = addr.clone().assume_checked().to_string();
                miner_change_amount_btc = detail.amount.to_btc();
            }
        }
    }

    let tx_fees_btc = tx_info.fee.unwrap_or_default().to_btc().abs();

    // 6. Mine confirmation block
    let conf_hashes = miner_rpc.generate_to_address(1, &miner_address)?;
    let block_hash = conf_hashes.first().expect("Failed to mine block");
    let block_height = miner_rpc.get_block_info(block_hash)?.height;

    // 7. Write out.txt (Integer style for amounts)
    let write_file = |path: &str| -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "{}", txid)?;
        writeln!(file, "{}", miner_input_address)?;
        writeln!(file, "{}", miner_input_amount_btc)?;
        writeln!(file, "{}", trader_output_address)?;
        writeln!(file, "{}", trader_output_amount_btc)?;
        writeln!(file, "{}", miner_change_address)?;
        writeln!(file, "{}", miner_change_amount_btc)?;
        writeln!(file, "-{}", tx_fees_btc)?;
        writeln!(file, "{}", block_height)?;
        writeln!(file, "{}", block_hash)
    };

    let _ = write_file("out.txt");
    let _ = write_file("../out.txt");

    Ok(())
}
