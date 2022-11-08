
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
    Principal,
};
use ic_cdk_macros::*;



#[update]
async fn public_key() -> Result<PublicKeyReply, String> {
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(),
    };
    let ic_canister_id = "aaaaa-aa";
    let ic = CanisterId::from_str(&ic_canister_id).unwrap();

    let caller = ic_cdk::caller().as_slice().to_vec();
    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![caller],
        key_id: key_id.clone(),
    };
    let (res,): (ECDSAPublicKeyReply,) = ic_cdk::call(ic, "ecdsa_public_key", (request,))
        .await
        .map_err(|e| format!("Failed to call ecdsa_public_key {}", e.1))?;

    Ok(PublicKeyReply {
        public_key: res.public_key,
    })
}



fn check_inclusion() {
    let request = CanisterHttpRequestArgument {
        url: url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        transform: Some(TransformType::Function(TransformFunc(candid::Func {
            principal: ic_cdk::api::id(),
            method: "transform".to_string(),
        }))),
        headers: request_headers,
    };

    let body = candid::utils::encode_one(&request).unwrap();
    ic_cdk::api::print(format!("Making IC http_request call {} now.", job));

    match ic_cdk::api::call::call_raw(
        Principal::management_canister(),
        "http_request",
        &body[..],
        2_000_000_000,
    )
    .await
    {
        Ok(result) => {
            // decode the result
            let decoded_result: HttpResponse =
                candid::utils::decode_one(&result).expect("IC http_request failed!");
            // put the result to hashmap
            FETCHED.with(|fetched| {
                let mut fetched = fetched.borrow_mut();
                let decoded_body = String::from_utf8(decoded_result.body)
                    .expect("Remote service response is not UTF-8 encoded.");
                decode_body_to_rates(&decoded_body, &mut fetched);
            });
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            ic_cdk::api::print(message.clone());

            // Since the remote request failed. Adding the de-queued job back again for retries.
            add_job_to_job_set(job);
        }
    }
}


#[update]
async fn mint(tx: String) -> () {
    // Decode tx, check "to" and "value"
    // Check if tx got included in block
    // mint token
} 