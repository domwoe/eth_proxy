use std::cell::RefCell;
use std::collections::HashMap;

use ic_cdk::{
    api::call::RejectionCode,
    export::{candid::Encode, Principal},
};

use ic_cdk::api::management_canister::main::*;

use ic_cdk::{print, storage};
use ic_cdk_macros::*;

const PROXY_WASM: &[u8] =
    std::include_bytes!("../../../target/wasm32-unknown-unknown/release/eth_proxy_backend.wasm");

thread_local! {
    static PROXIES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::default());
}

fn err_to_string(err: (RejectionCode, String)) -> String {
    format!("{:?}: {}", err.0, err.1)
}

#[update]
async fn create_proxy(eth_address: String) -> Result<Principal, String> {
    print("creating new proxy canister...");

    let create_arg = CreateCanisterArgument::default();

    let cycles = 1000000000000;

    let canister_id = create_canister_with_extra_cycles(create_arg, cycles)
        .await
        .map_err(|err| err_to_string(err))?
        .0
        .canister_id;

    let init_arg = Encode!(&eth_address).map_err(|err| format!("{:?}", err))?;

    let install_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: PROXY_WASM.to_vec(),
        arg: init_arg,
    };
    install_code(install_arg)
        .await
        .map_err(|err| err_to_string(err))?;

    // Make proxy canister its own controller
    let mut settings = CanisterSettings::default();
    settings.controllers = Some(vec![canister_id]);
    let update_arg = UpdateSettingsArgument {
        canister_id,
        settings,
    };

    update_settings(update_arg)
        .await
        .map_err(|err| err_to_string(err))?;

    Ok(canister_id)
}

#[query]
async fn get_proxy(eth_address: String) -> Result<String, String> {
    let proxy = PROXIES.with(|p| p.borrow().get(&eth_address).map(|s| s.to_string()));

    let proxy = proxy.map(|s| s.to_string());

    proxy.ok_or(String::from("No proxy found."))
}

#[pre_upgrade]
fn pre_upgrade() {
    PROXIES.with(|p| storage::stable_save((p,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let (proxies,): (HashMap<String, String>,) = storage::stable_restore().unwrap();
    PROXIES.with(|p| *p.borrow_mut() = proxies);
}
