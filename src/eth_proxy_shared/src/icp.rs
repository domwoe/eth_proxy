use candid::Principal;
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, Memo, Subaccount, Tokens,
    TransferArgs, DEFAULT_FEE,
};

const ETH_TO_ICP_DECIMALS: u64 = 10 * 1000 * 1000 * 1000;

pub async fn get_balance(ledger_id: Principal, address: &str) -> String {
    let subaccount = address_to_subaccount(address);

    let account = AccountIdentifier::new(&ic_cdk::id(), &subaccount);

    let args = AccountBalanceArgs { account };

    let zero = Tokens::from_e8s(0);

    let balance = account_balance(ledger_id, args).await.unwrap_or(zero);

    format!("0x{:X}", balance.e8s() * ETH_TO_ICP_DECIMALS)
}

pub async fn transfer(ledger_id: Principal, from: &str, to: &str, amount: u64) {
    let from_subaccount = address_to_subaccount(from);
    let to_subaccount = address_to_subaccount(to);

    let account = AccountIdentifier::new(&ic_cdk::id(), &to_subaccount);

    // let icp_ledger = CONF.with(|conf| conf.borrow().ledger_canister_id);

    let _block = ic_ledger_types::transfer(
        ledger_id,
        TransferArgs {
            memo: Memo(0),
            amount: Tokens::from_e8s(amount / ETH_TO_ICP_DECIMALS),
            fee: DEFAULT_FEE,
            from_subaccount: Some(from_subaccount),
            to: account,
            created_at_time: None,
        },
    )
    .await
    .expect("call to ledger failed")
    .unwrap();
}

pub fn address_to_subaccount(address: &str) -> Subaccount {
    let address = address.strip_prefix("0x").unwrap();

    let addr_bytes = hex::decode(address).unwrap();

    let mut padded = [0; 32];
    padded[..20].copy_from_slice(&addr_bytes[..]);

    Subaccount(padded)
}
