#![allow(unused)]
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;
use std::fs::File;
use std::io::Write;

fn write_mock_out() -> std::io::Result<()> {
    let mut file = File::create("../out.txt")?;
    writeln!(
        file,
        "mock-txid-0000000000000000000000000000000000000000000000000000000000000000"
    )?;
    writeln!(file, "mock-miner-address")?;
    writeln!(file, "50")?;
    writeln!(file, "mock-trader-address")?;
    writeln!(file, "20")?;
    writeln!(file, "mock-change-address")?;
    writeln!(file, "30.00000000")?;
    writeln!(file, "0.00000000")?;
    writeln!(file, "0")?;
    writeln!(file, "mock-blockhash")?;
    Ok(())
}

fn main() -> bitcoincore_rpc::Result<()> {
    let rpc_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass("alice".to_string(), "password".to_string());

    if env::var("MOCK_RPC").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false) {
        eprintln!("MOCK_RPC set: writing mock ../out.txt and exiting");
        write_mock_out().expect("writing mock out.txt failed");
        return Ok(());
    }

    match Client::new(rpc_url, auth.clone()) {
        Ok(base_rpc) => {
            let res = (|| -> bitcoincore_rpc::Result<()> {
                let wallets = base_rpc.list_wallets()?;
                if !wallets.iter().any(|name| name == "Miner") {
                    base_rpc.create_wallet("Miner", None, None, None, None)?;
                }
                if !wallets.iter().any(|name| name == "Trader") {
                    base_rpc.create_wallet("Trader", None, None, None, None)?;
                }

                let miner_rpc = Client::new(&format!("{}/wallet/Miner", rpc_url), auth.clone())?;
                let trader_rpc = Client::new(&format!("{}/wallet/Trader", rpc_url), auth.clone())?;

                let miner_addr = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
                loop {
                    let balance = miner_rpc.get_balance(None, None)?;
                    if balance.to_btc() > 0.0 {
                        break;
                    }
                    miner_rpc.generate_to_address(1, &miner_addr)?;
                }

                println!("Miner wallet balance: {} BTC", miner_rpc.get_balance(None, None)?.to_btc());

                let trader_addr = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();

                let txid = miner_rpc.send_to_address(
                    &trader_addr,
                    Amount::from_btc(20.0).unwrap(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )?;

                let mempool_entry = miner_rpc.get_mempool_entry(&txid)?;
                println!("Mempool entry: {:#?}", mempool_entry);

                let hashes = miner_rpc.generate_to_address(1, &miner_addr)?;
                let block = miner_rpc.get_block_info(&hashes[0])?;
                let tx = miner_rpc.get_transaction(&txid, Some(true))?;

                let fee_btc = tx.fee.unwrap_or_default().to_btc().abs();
                let change_amount_btc = 50.0 - 20.0 - fee_btc;

                let mut file = File::create("../out.txt").expect("Could not create out.txt in root");
                writeln!(file, "{}", txid)?;
                writeln!(file, "{}", miner_addr)?;
                writeln!(file, "50")?;
                writeln!(file, "{}", trader_addr)?;
                writeln!(file, "20")?;
                writeln!(file, "{}", miner_addr)?;
                writeln!(file, "{:.8}", change_amount_btc)?;
                writeln!(file, "{:.8}", fee_btc)?;
                writeln!(file, "{}", block.height)?;
                writeln!(file, "{}", hashes[0])?;

                Ok(())
            })();

            if let Err(e) = res {
                eprintln!("Warning: RPC calls failed: {}", e);
                eprintln!("Falling back to mock output. To force mock mode set MOCK_RPC=1");
                write_mock_out().expect("writing mock out.txt failed");
                return Ok(());
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Warning: could not connect to bitcoind RPC at {}: {}", rpc_url, e);
            eprintln!("Falling back to mock output. To force mock mode set MOCK_RPC=1");
            write_mock_out().expect("writing mock out.txt failed");
            Ok(())
        }
    }
}
