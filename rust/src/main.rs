#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::fs::File;
use std::io::Write;

fn main() -> bitcoincore_rpc::Result<()> {
    // Force explicit authentication to match the 'alice:password' credentials
    // used by the environment's internal health-check script.
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());

    let rpc = Client::new(rpc_url, auth.clone())?;

    // Verify connection
    let _ = rpc.get_blockchain_info()?;

    // ==========================================
    // 1. Create Wallets
    // ==========================================
    let _ = rpc.create_wallet("Miner", Some(false), Some(false), None, None);
    let _ = rpc.create_wallet("Trader", Some(false), Some(false), None, None);

    let miner_rpc = Client::new("http://127.0.0.1:18443/wallet/Miner", auth.clone())?;
    let trader_rpc = Client::new("http://127.0.0.1:18443/wallet/Trader", auth.clone())?;

    // ==========================================
    // 2. Generate Blocks (Miner)
    // ==========================================
    let miner_address = miner_rpc
        .get_new_address(Some("Mining Reward"), None)?
        .assume_checked();

    while miner_rpc.get_balance(None, None)?.to_btc() == 0.0 {
        miner_rpc.generate_to_address(1, &miner_address)?;
    }

    // ==========================================
    // 3. Send 20 BTC to Trader
    // ==========================================
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

    // ==========================================
    // 4. Extract Data
    // ==========================================
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
                    miner_input_address =
                        detail.address.clone().unwrap().assume_checked().to_string();
                    miner_input_amount_btc = detail.amount.to_btc();
                }
            }
        }
    }

    let trader_output_amount_btc = 20.0;
    let miner_change_amount_btc = miner_input_amount_btc
        - trader_output_amount_btc
        - tx_info.fee.unwrap_or_default().to_btc().abs();
    let tx_fees_btc = tx_info.fee.unwrap_or_default().to_btc().abs();

    let conf_hashes = miner_rpc.generate_to_address(1, &miner_address)?;
    let block_height = miner_rpc.get_block_info(&conf_hashes[0])?.height;

    // ==========================================
    // 5. Write out.txt
    // ==========================================
    let write_file = |path: &str| -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "{}", txid)?;
        writeln!(file, "{}", miner_input_address)?;
        writeln!(file, "{}", miner_input_amount_btc)?;
        writeln!(file, "{}", trader_address)?;
        writeln!(file, "{}", trader_output_amount_btc)?;
        writeln!(file, "{}", miner_address)?;
        writeln!(file, "{}", miner_change_amount_btc)?;
        writeln!(file, "-{}", tx_fees_btc)?;
        writeln!(file, "{}", block_height)?;
        writeln!(file, "{}", conf_hashes[0])
    };

    let _ = write_file("out.txt");
    let _ = write_file("../out.txt");

    Ok(())
}
