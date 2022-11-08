use candid::{CandidType, Deserialize, Func, Principal};
use ethers_core::types::TransactionReceipt;
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;
use serde::Serialize;
use serde_bytes::{ByteBuf, Bytes};
use serde_json::*;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static __STATE: RefCell<Option<ProxyState>> = RefCell::default();
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProxyState {
    pub transactions: BTreeMap<String, TransactionReceipt>,

    pub block_number: u64,

    pub tx_count: BTreeMap<String, u64>,
}

impl Default for ProxyState {
    fn default() -> Self {
        ProxyState {
            transactions: BTreeMap::default(),
            block_number: 0,
            tx_count: BTreeMap::default(),
        }
    }
}

/// Take the current state.
///
/// After calling this function the state won't be initialized anymore.
/// Panics if there is no state.
pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(ProxyState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut ProxyState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&ProxyState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: ProxyState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub ledger_canister_id: Principal,
}

impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
        }
    }
}

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
    pub status_code: u16,
    pub upgrade: Option<bool>,
    pub headers: Vec<HeaderField>,
    pub body: Cow<'static, Bytes>,
    pub streaming_strategy: Option<StreamingStrategy>,
}
