use candid::Principal;
use std::str::FromStr;

pub async fn transfer(token: &str, to: &str, amount: u64) {}


pub async fn get_symbol(token: &str) -> String {
    let res: ic_cdk::api::call::CallResult<(String,)> =
        ic_cdk::call(Principal::from_str(token).unwrap(), "symbol", ()).await;

    match res {
        Ok((symbol,)) => symbol,
        Err((code, err)) => {
            ic_cdk::print(format!("Error with Code: {:?} and message {}", code, err));
            // format!("0x{}", hex::encode("OGY"))
            String::from("OGY")
        }
    }
}

pub async fn get_decimals(token: &str) -> String {
    format!("0x{:X}", 13)
}

pub async fn get_balance(token: &str) -> String {
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