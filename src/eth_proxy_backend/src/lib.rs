use candid::{CandidType, Deserialize, Func, Principal};

use ethers_core::abi::encode;
use ethers_core::types::{Address, Bloom, Log, Transaction, TransactionReceipt, H256, U256, U64};
use ethers_core::{abi, utils::rlp};

use ic_cdk::storage;
use ic_cdk_macros::*;

use hex;

use phf::phf_map;

use serde::Serialize;
use serde_bytes::{ByteBuf, Bytes};
use serde_json::*;

use std::borrow::{Cow};

use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

pub type HeaderField = (String, String);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseBlock {
    pub base_fee_per_gas: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<serde_json::Value>,
    pub id: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Value,
    pub id: serde_json::Value,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Token {}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback { callback: Func, token: Token },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    pub body: ByteBuf,
    pub token: Option<Token>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    status_code: u16,
    upgrade: Option<bool>,
    headers: Vec<HeaderField>,
    body: Cow<'static, Bytes>,
    streaming_strategy: Option<StreamingStrategy>,
}

thread_local! {
    //TODO: Make TRANSACTIONS stable
    static TRANSACTIONS: RefCell<HashMap<String, TransactionReceipt>> = RefCell::new(HashMap::default());
    static ERC20_ABI: abi::Abi = serde_json::from_str(include_str!("erc20.json")).unwrap();
    // stable
    static BLOCK_NUMBER: RefCell<u64> = RefCell::new(9999);
    static TXN_COUNT: RefCell<u64> = RefCell::new(0);

    static ETH_ACCOUNT: RefCell<String> = RefCell::new("".to_string());
    static TOKENS: phf::Map<&'static str, &'static str> = phf_map! {
        // WXTC
        "0x71c7656ec7ab88b098defb751b7401b5f6d89761" => "aanaa-xaaaa-aaaah-aaeiq-cai",
        // WICP
        "0x71c7656ec7ab88b098defb751b7401b5f6d89763" => "utozz-siaaa-aaaam-qaaxq-cai",
        // OGY
        "0x71c7656ec7ab88b098defb751b7401b5f6d89764" => "jwcfb-hyaaa-aaaaj-aac4q-cai",
    };

}

#[init]
fn init(eth_account: String) {

    ETH_ACCOUNT.with(|account| *account.borrow_mut() = eth_account );

}

// We increase the block number everytime it's called. This should probaby be optimized
// at some point
fn block_number() -> u64 {
    BLOCK_NUMBER.with(|n| *n.borrow_mut() += 1);
    BLOCK_NUMBER.with(|n| n.borrow().clone())
}

// We increase the transaction count everytime it's called. This is used as the nonce
// by MetaMask
fn txn_count() -> String {
    TXN_COUNT.with(|n| *n.borrow_mut() += 1);

    let n = BLOCK_NUMBER.with(|n| n.borrow().clone());

    format!("0x{:X}", n)
}

fn store_receipt(hash: String, txn: Transaction, input: Vec<abi::Token>) {
    ic_cdk::print(format!("Store receipt for tx: {}", &hash));

    let event = ERC20_ABI.with(|c| c.event("Transfer").cloned().unwrap());

    // Indexed arguments

    let topics: Vec<H256> = vec![
        event.signature(),
        H256::from(Address::from(txn.from)), // from
        H256::from(input[0].clone().into_address().unwrap()), // to
    ];

    // Unindexed arguments
    let data = vec![input[1].clone()]; // amount

    let log = Log {
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
    };

    let logs = vec![log];

    let txn_receipt = TransactionReceipt {
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

    TRANSACTIONS.with(|s| {
        s.borrow_mut().insert(hash, txn_receipt);
    });
}

fn get_receipt(hash: &str) -> Value {
    ic_cdk::print(format!("Getting receipt for tx: {}", hash));

    let txn_receipt = TRANSACTIONS
        .with(|s| s.borrow().get(hash).cloned())
        .expect("Not found");

    serde_json::to_value(txn_receipt).unwrap()
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

async fn dip20_transfer(token: &str, to: &str, amount: u64) {}

async fn get_dip20_symbol(token: &str) -> String {
    let res: ic_cdk::api::call::CallResult<(String,)> =
        ic_cdk::call(Principal::from_str(token).unwrap(), "symbol", ()).await;

    match res {
        Ok((symbol,)) => symbol.to_string(),
        Err((code, err)) => {
            ic_cdk::print(format!("Error with Code: {:?} and message {}", code, err));
            // format!("0x{}", hex::encode("OGY"))
            String::from("OGY")
        }
    }
}

async fn get_dip20_decimals(token: &str) -> String {
    format!("0x{:X}", 13)
}

async fn get_dip20_balance(token: &str) -> String {
    let res: ic_cdk::api::call::CallResult<(candid::Nat,)> = ic_cdk::call(
        Principal::from_str(token).unwrap(),
        "balanceOf",
        (ic_cdk::id(),),
    )
    .await;

    // TODO: Proper conversion to hex
    let balance = match res {
        Ok((balance,)) => balance,
        Err((code, err)) => {
            ic_cdk::print(format!("Error with Code: {:?} and message {}", code, err));
            let n: u64 = 99999999999999;
            candid::Nat::from(n)
        }
    };

    format!("0x{:X}", balance.0)
}

fn get_cycles_balance() -> String {
    let balance: u128 = ic_cdk::api::canister_balance128();
    format!("0x{:X}", balance)
}

async fn process_txn(txn: &str) -> String {
    ic_cdk::print("Processing transaction...");

    ic_cdk::print(format!("{}", txn));

    let raw_txn = &txn
        .parse::<ethers_core::types::Bytes>()
        .expect("unable to parse raw tx");

    let decoded = rlp::decode::<Transaction>(raw_txn).expect("Could not decode tx");

    //TODO: Actually verify the signature of the transaction and the nonce
    ic_cdk::print(format!("{:#?}", &decoded));

    let data = decoded.input.to_string();
    let data = data.strip_prefix("0x").unwrap();

    ic_cdk::print(format!("{:#?}", data));

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
                    store_receipt(txn_hash.clone(), decoded, input);
                    ic_cdk::print(&txn_hash);
                    txn_hash
                }
                _ => String::from(""),
            }
        }
        None => String::from(""),
    }
}

async fn parse_call(data: Vec<Value>) -> String {
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
                            "balanceOf" => get_dip20_balance("aanaa-xaaaa-aaaah-aaeiq-cai").await,
                            "symbol" => get_dip20_symbol("aanaa-xaaaa-aaaah-aaeiq-cai").await,
                            "decimals" => get_dip20_decimals("aanaa-xaaaa-aaaah-aaeiq-cai").await,
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
    let bytes = req.body.clone().into_vec();
    let rpc_msg = std::str::from_utf8(&bytes).unwrap();

    let req: RpcRequest = serde_json::from_str(&rpc_msg).unwrap();

    ic_cdk::print(format!("{:#?}", req));

    let (result, upgrade) = match req.method.as_str() {
        "eth_call" => (Value::from(String::from("")), true),
        "net_version" => (Value::from(String::from("255")), false),
        "eth_chainId" => (Value::from(String::from("255")), false),
        "eth_blockNumber" => (Value::from(String::from("")), true),
        "eth_estimateGas" => (Value::from(format!("{:X}", 210000)), false),
        "eth_gasPrice" => (Value::from(format!("{:X}", 0)), false),
        "eth_getBlockByNumber" => (Value::from(get_block()), false),
        "eth_getBlockByHash" => (Value::from(get_block()), false),
        "eth_getBalance" => (Value::from(get_cycles_balance()), false),
        "eth_getTransactionCount" => (Value::from(txn_count()), false),
        "eth_getTransactionReceipt" => (
            Value::from(get_receipt(req.params[0].as_str().unwrap())),
            false,
        ),
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

    let req: RpcRequest = serde_json::from_str(&rpc_msg).unwrap();

    let result = match req.method.as_str() {
        "eth_blockNumber" => Value::from(block_number()),
        "eth_call" => Value::from(parse_call(req.params).await),
        "eth_sendRawTransaction" => {
            ic_cdk::print("First step in processing a raw tx");
            let raw_tx = req.params[0].as_str().unwrap();
            Value::from(process_txn(raw_tx).await)
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

#[pre_upgrade]
fn pre_upgrade() {
    BLOCK_NUMBER.with(|n| storage::stable_save((n,)).unwrap());
    ETH_ACCOUNT.with(|a| storage::stable_save((a,)).unwrap());
    TXN_COUNT.with(|n| storage::stable_save((n,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let (block_number,): (u64,) = storage::stable_restore().unwrap();
    BLOCK_NUMBER.with(|n| *n.borrow_mut() = block_number);

    let (eth_account,): (String,) = storage::stable_restore().unwrap();
    ETH_ACCOUNT.with(|a| *a.borrow_mut() = eth_account);


    let (txn_count,): (u64,) = storage::stable_restore().unwrap();
    TXN_COUNT.with(|n| *n.borrow_mut() = txn_count);
}
