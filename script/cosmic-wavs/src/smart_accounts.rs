use cosmwasm_std::to_json_binary;

#[cosmwasm_schema::cw_serde]
pub struct CosmwasmAuthenticatorInitData {
    pub contract: String,
    pub params: Vec<u8>,
}

#[cosmwasm_schema::cw_serde]
pub struct TxExtension {
    pub selected_authenticators: Vec<u64>,
}

#[cosmwasm_schema::cw_serde]
pub struct MsgAddAuthenticator {
    pub sender: String,
    pub authenticator_type: String,
    pub data: Vec<u8>,
}

/// Register a given seckp256k1 key with a specific authenticator
pub fn setup_wavs_smart_account(
    chain: &str,
    authenticator: MsgAddAuthenticator,
) -> Result<prost_types::Any, anyhow::Error> {
    let type_url = match chain {
        "osmosis" => "/osmosis.smartaccount.v1beta1.MsgAddAuthenticator".to_string(),
        "bitsong" => "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".to_string(),
        _ => panic!("bad chain type"),
    };
    // register custom authenticator to account
    Ok(prost_types::Any { type_url, value: to_json_binary(&authenticator)?.to_vec() })
}

/// Generated the object for using in a msg to register an account with a smart contract authenticator
pub fn default_msg_add_authenticator_wasm(
    sender: String,
    contract: String,
    params: Vec<u8>,
) -> Result<MsgAddAuthenticator, anyhow::Error> {
    // register custom authenticator to account
    Ok(MsgAddAuthenticator {
        sender,
        authenticator_type: "CosmwasmAuthenticatorV1".into(),
        data: to_json_binary(&CosmwasmAuthenticatorInitData { contract, params })?.to_vec(),
    })
}
