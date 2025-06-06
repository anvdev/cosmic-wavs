use cosmwasm_std::to_json_binary;

#[cosmwasm_schema::cw_serde]
pub struct CosmwasmAuthenticatorInitData {
    pub contract: String,
    pub params: Vec<u8>,
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
