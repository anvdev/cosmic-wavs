


#[cosmwasm_schema::cw_serde]
pub struct WavsBlsCosmosActionAuth {
    /// b2 point for operator public key
    pub pubkey_g2: String,
    /// base64 encoded sha256sum hash of msg being signed
    pub base64_msg_hash: String,
    /// msg that bls12 private key is signing
    pub msg: Vec<u8>,
    ///
    pub signature: String,
}