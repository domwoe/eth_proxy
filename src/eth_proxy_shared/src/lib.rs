mod dip20;
mod icp;
mod types;

use candid::candid_method;
use ethers_core::abi::encode;
use ethers_core::types::{Address, Bloom, Log, Transaction, TransactionReceipt, H256, U256, U64};
use ethers_core::{abi, utils::rlp};

use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_cdk_macros::*;

use hex;

use phf::phf_map;

use serde_json::*;

use std::borrow::Cow;
use std::cell::RefCell;

use types::*;

// TODO:
// - Frontend
// - Block Number
// - Sign Delegation
// - Get Principal from ETH Address
// - ERC 721

thread_local! {
    static CONF: RefCell<Conf> = RefCell::new(Conf::default());
    static ERC20_ABI: abi::Abi = serde_json::from_str(include_str!("erc20.json")).unwrap();
    static TOKENS: phf::Map<&'static str, &'static str> = phf_map! {
        // WXTC
        "0x71c7656ec7ab88b098defb751b7401b5f6d89761" => "aanaa-xaaaa-aaaah-aaeiq-cai",
        // WICP
        "0x71c7656ec7ab88b098defb751b7401b5f6d89763" => "utozz-siaaa-aaaam-qaaxq-cai",
        // OGY
        "0x71c7656ec7ab88b098defb751b7401b5f6d89764" => "jwcfb-hyaaa-aaaaj-aac4q-cai",
        // ckETH
        "0x71c7656ec7ab88b098defb751b7401b5f6d89765" => "jwcfb-hyaaa-aaaaj-aac4q-cai",
    };

}

#[init]
#[candid_method(init)]
fn init(conf: Conf) {
    CONF.with(|c| c.replace(conf));
    replace_state(ProxyState::default());
}

// We increase the block number everytime it's called. This should probaby be optimized
// at some point
fn block_number() -> u64 {
    mutate_state(|s| s.block_number += 1);
    read_state(|s| s.block_number)
}

fn txn_count(address: String) -> String {
    let n = read_state(|s| {
        s.tx_count
            .get(&address)
            .map(|n| n.to_owned())
            .unwrap_or(0 as u64)
    });

    format!("0x{:X}", n)
}

fn store_receipt(hash: String, txn: Transaction, input: Option<Vec<abi::Token>>) {
    ic_cdk::print(format!("Store receipt for tx: {}", &hash));

    let log = match input {
        Some(input) => {
            let event = ERC20_ABI.with(|c| c.event("Transfer").cloned().unwrap());

            // Indexed arguments

            let topics: Vec<H256> = vec![
                event.signature(),
                H256::from(Address::from(txn.from)), // from
                H256::from(input[0].clone().into_address().unwrap()), // to
            ];

            // Unindexed arguments
            let data = vec![input[1].clone()]; // amount

            Log {
                address: txn.to.unwrap(),
                topics,
                data: ethers_core::types::Bytes::from(encode(&data[..])),
                block_hash: None,
                block_number: None,
                transaction_hash: None,
                transaction_index: None,
                log_index: None,
                transaction_log_index: None,
                log_type: None,
                removed: None,
            }
        }
        None => Log::default(),
    };

    let logs = vec![log];

    let tx_receipt = TransactionReceipt {
        transaction_hash: txn.hash(),
        transaction_index: ethers_core::types::U64::default(),
        block_hash: None,
        block_number: Some(U64::from(block_number())),
        from: txn.from,
        to: txn.to,
        cumulative_gas_used: ethers_core::types::U256::from(0),
        gas_used: Some(U256::from(0)),
        contract_address: None,
        logs,
        status: Some(U64::from(1)),
        root: None,
        logs_bloom: Bloom::default(),
        transaction_type: None,
        effective_gas_price: None,
    };

    mutate_state(|s| s.transactions.insert(hash, tx_receipt));

}

fn get_receipt(hash: &str) -> Value {
    ic_cdk::print(format!("Getting receipt for tx: {}", hash));

    let tx_receipt = read_state(|s| s.transactions.get(hash).cloned());

    serde_json::to_value(tx_receipt).unwrap()
}

fn get_block() -> Value {
    let timestamp = ic_cdk::api::time() / 1000000000;

    let block = BaseBlock {
        base_fee_per_gas: format!("0"),
        gas_limit: format!("0"),
        gas_used: format!("0"),
        timestamp: format!("{}", hex::encode(timestamp.to_string())),
    };

    serde_json::to_value(block).unwrap()
}

async fn process_tx(tx: &str) -> String {
    ic_cdk::print("Processing transaction...");

    // Decode transaction from hex
    let mut decoded = rlp::decode::<Transaction>(
        &tx.parse::<ethers_core::types::Bytes>()
            .expect("unable to parse raw tx"),
    )
    .expect("unable to decode raw tx");

    ic_cdk::print(format!("{:#?}", &decoded));

    // Recover address from signature
    // This is also used to be sure that the proper private key signed the tx
    decoded.recover_from_mut().unwrap();

    let recipient = format!("0x{}", hex::encode(decoded.to.unwrap().as_bytes()));
    let from = format!("0x{}", hex::encode(decoded.from.as_bytes()));
    let amount = decoded.value.as_u64();

    // Check nonce
    let tx_count = read_state(|s| s.tx_count.get(&from).cloned());
   
    if let Some(n) = tx_count {
        if decoded.nonce.as_u64() <= n {
            ic_cdk::trap("Nonce too small");
        }
    };

    decoded.hash = decoded.hash();

    // If amount > 0 then interpret as ICP transaction
    let tx_id = if amount > 0 {
        let ledger_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
        icp::transfer(ledger_id, &from, &recipient, amount).await;
        let txn_hash = format!("0x{}", hex::encode(decoded.hash().as_bytes()));
        store_receipt(txn_hash.clone(), decoded, None);
        txn_hash
    // Else, interpret as token transfer
    } else {
        let data = decoded.input.to_string();
        let data = data.strip_prefix("0x").unwrap();

        let method_id = &data[0..8];

        let f = ERC20_ABI.with(|c| {
            ic_cdk::print(format!("{:?}", method_id));
            c.functions()
                .into_iter()
                .find(|f| hex::encode(f.short_signature()) == method_id)
                .cloned()
        });

        match f {
            Some(f) => {
                ic_cdk::print(&f.name);

                let input = f.decode_input(&hex::decode(&data[8..]).unwrap()).unwrap();

                ic_cdk::print(format!("{:?}", input));

                match f.name.as_str() {
                    "transfer" => {
                        ic_cdk::print("Transfering tokens..");
                        // TODO: Implement token transfer
                        // We fake this for now
                        // dip20_transfer("aanaa-xaaaa-aaaah-aaeiq-cai", ).await
                        let txn_hash = format!("0x{}", hex::encode(decoded.hash().as_bytes()));
                        store_receipt(txn_hash.clone(), decoded, Some(input));
                        ic_cdk::print(&txn_hash);
                        txn_hash
                    }
                    _ => String::from(""),
                }
            }
            None => String::from(""),
        }
    };

    // Increase transaction count for address. This is used as the nonce by MetaMask
    mutate_state(|s| *s.tx_count.entry(from).or_insert(0 as u64) += 1);

    tx_id
}

async fn process_call(data: Vec<Value>) -> String {
    ic_cdk::print("Processing call...");
    match &data[0] {
        Value::Object(obj) => {
            if obj.contains_key("data") {
                let data_w_prefix = obj["data"].as_str().unwrap();
                let data = data_w_prefix.strip_prefix("0x").unwrap();

                let method_id = &data[0..8];

                let f = ERC20_ABI.with(|c| {
                    ic_cdk::print(format!("{:?}", method_id));
                    c.functions()
                        .into_iter()
                        .find(|f| hex::encode(f.short_signature()) == method_id)
                        .cloned()
                });
                match f {
                    Some(f) => {
                        ic_cdk::print(&f.name);

                        // TODO: Need to properly transform eth-style token address to principal to token canister
                        match f.name.as_str() {
                            "balanceOf" => dip20::get_balance("aanaa-xaaaa-aaaah-aaeiq-cai").await,
                            "symbol" => dip20::get_symbol("aanaa-xaaaa-aaaah-aaeiq-cai").await,
                            "decimals" => dip20::get_decimals("aanaa-xaaaa-aaaah-aaeiq-cai").await,
                            _ => String::from(""),
                        }
                    }
                    None => String::from(""),
                }
            } else {
                String::from("")
            }
        }
        _ => String::from(""),
    }
}

#[query]
pub fn http_request(req: HttpRequest) -> HttpResponse {
    let bytes = req.body.into_vec();
    let rpc_msg = std::str::from_utf8(&bytes).unwrap();

    let req: RpcRequest = serde_json::from_str(rpc_msg).unwrap();

    ic_cdk::print(format!("{:#?}", req));

    let (result, upgrade) = match req.method.as_str() {
        "eth_call" => (Value::from(String::from("")), true),
        "net_version" => (Value::from(String::from("255")), false),
        "eth_chainId" => (Value::from(String::from("255")), false),
        "eth_blockNumber" => (Value::from(String::from("")), true),
        "eth_estimateGas" => (Value::from(format!("{:X}", 210000)), false),
        "eth_gasPrice" => (Value::from(format!("{:X}", 0)), false),
        "eth_getBlockByNumber" => (get_block(), false),
        "eth_getBlockByHash" => (get_block(), false),
        "eth_getBalance" => (Value::from(""), true),
        "eth_getTransactionCount" => (Value::from(txn_count(req.params[0].to_string())), false),
        "eth_getTransactionReceipt" => (get_receipt(req.params[0].as_str().unwrap()), false),
        "eth_sendRawTransaction" => (Value::from(String::from("")), true),
        _ => (Value::from(String::from("")), false),
    };

    let resp = serde_json::to_string(&RpcResponse {
        jsonrpc: String::from("2.0"),
        result,
        id: req.id,
    })
    .unwrap();

    if !upgrade {
        ic_cdk::print(format!("{:#?}", resp));
    }

    HttpResponse {
        status_code: 200,
        upgrade: Some(upgrade),
        headers: [(
            String::from("content-type"),
            String::from("application/json"),
        )]
        .to_vec(),
        body: Cow::Owned(serde_bytes::ByteBuf::from(resp.as_bytes())),
        streaming_strategy: None,
    }
}

#[update]
pub async fn http_request_update(req: HttpRequest) -> HttpResponse {
    let bytes = req.body.clone().into_vec();
    let rpc_msg = std::str::from_utf8(&bytes).unwrap();

    let req: RpcRequest = serde_json::from_str(rpc_msg).unwrap();

    let result = match req.method.as_str() {
        "eth_blockNumber" => Value::from(block_number()),
        "eth_call" => Value::from(process_call(req.params).await),
        "eth_getBalance" => {
            let ledger_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
            Value::from(icp::get_balance(ledger_id, req.params[0].as_str().unwrap()).await)
        }
        "eth_sendRawTransaction" => {
            ic_cdk::print("First step in processing a raw tx");
            let raw_tx = req.params[0].as_str().unwrap();
            Value::from(process_tx(raw_tx).await)
        }
        _ => Value::from(String::from("")),
    };

    let resp = serde_json::to_string(&RpcResponse {
        jsonrpc: String::from("2.0"),
        result,
        id: req.id,
    })
    .unwrap();

    ic_cdk::print(format!("{:#?}", resp));

    HttpResponse {
        status_code: 200,
        upgrade: None,
        headers: [(
            String::from("content-type"),
            String::from("application/json"),
        )]
        .to_vec(),
        body: Cow::Owned(serde_bytes::ByteBuf::from(resp.as_bytes())),
        streaming_strategy: None,
    }
}

#[query]
pub fn get_account(address: String) -> String {
    let subaccount = icp::address_to_subaccount(&address);

    let account = ic_ledger_types::AccountIdentifier::new(&ic_cdk::id(), &subaccount);

    account.to_string()
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("Executing pre upgrade");
    ciborium::ser::into_writer(&take_state(|s| s), StableWriter::default())
        .expect("failed to encode proxy state");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Executing post upgrade");
    replace_state(
        ciborium::de::from_reader(StableReader::default()).expect("failed to proxy state"),
    );
}
