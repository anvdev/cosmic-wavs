
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
async fn setup_bitsong_smart_account(
    authenticator: MsgAddAuthenticator,
) -> Result<prost_types::Any, anyhow::Error> {
    // register custom authenticator to account
    Ok(prost_types::Any {
        type_url: "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".into(),
        value: to_json_binary(&authenticator)?.to_vec(),
    })
}
