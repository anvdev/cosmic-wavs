use commonware_codec::extensions::DecodeExt;
use commonware_cryptography::Signer;
use commonware_cryptography::{bls12381::PublicKey, Bls12381};
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, SignerInfo, Tx, TxBody};
use cosmos_sdk_proto::Any;
use cosmwasm_std::to_json_binary;
use sha2::{Digest, Sha256};

use crate::smart_accounts::TxExtension;

pub const TX_EXTENSION_TYPE: &str = "/bitsong.smartaccount.v1beta1.TxExtension";

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

/// Register a given seckp256k1 key with a specific authenticator
pub async fn form_wavs_tx(
    tx_body: TxBody,
    gas_to_use: u64,
    signer_infos: Vec<SignerInfo>,
    signatures: Vec<Vec<u8>>,
) -> Result<Tx, anyhow::Error> {
    Ok(Tx {
        body: Some(tx_body),
        auth_info: Some(AuthInfo {
            signer_infos,
            fee: Some(Fee {
                amount: vec![Coin { denom: "ubtsg".into(), amount: 40_000u64.to_string() }],
                gas_limit: gas_to_use * 2,
                payer: "".to_string(), // wavs operated account
                granter: "".to_string(),
            }),
            tip: None,
        }),
        signatures, // added array of bls signatures
    })
}

/// Register a given seckp256k1 key with a specific authenticator
pub fn form_smart_account_msg(
    mut imported_signer: Bls12381,
    cosmic_wavs_actions: &Vec<Any>,
) -> Result<([u8; 32], Vec<u8>), anyhow::Error> {
    // create sha256sum bytes that are being signed by operators for aggregated approval.
    // current implementation signs binary formaated array of Any msgs being authorized.
    let msg_digest: [u8; 32] =
        Sha256::digest(to_json_binary(cosmic_wavs_actions)?.as_ref()).to_vec().try_into().unwrap();
    // let namespace = Some(&b"additional_namespace. Commonware library already generates hash with standard dst"[..]);
    let signature = imported_signer.sign(None, &msg_digest).to_vec();

    // register custom authenticator to account
    Ok((msg_digest, signature))
}

pub fn get_smart_account(wavs_bls_sk: String) -> Result<Bls12381, anyhow::Error> {
    // Import the bls12-381 private key
    let bls_key_pair = match <Bls12381 as commonware_cryptography::Signer>::PrivateKey::decode(
        hex::decode(wavs_bls_sk.as_bytes())?.as_ref(),
    ) {
        Ok(key) => key,
        Err(e) => {
            return Err(e.into());
        }
    };
    // Create a signer from the imported key
    Ok(<Bls12381 as commonware_cryptography::Signer>::from(bls_key_pair)
        .expect("broken private key"))
}

/// Register a SignerInfo
pub fn get_smart_acount_signer_info(pk: &PublicKey) -> SignerInfo {
    SignerInfo {
        public_key: Some(Any {
            type_url: "/cosmos.crypto.bls12_381.PubKey".into(),
            value: pk.to_vec(),
        }),
        mode_info: None,
        sequence: 0,
    }
}

/// Form cosmos-sdk-proto TxBody for smart account actions
pub async fn form_smart_acccount_tx_body(
    current_height: u64,
    cosmic_wavs_actions: Vec<Any>,
    selected_authenticators: Vec<u64>,
) -> Result<TxBody, anyhow::Error> {
    Ok(TxBody {
        messages: cosmic_wavs_actions,
        memo: "Cosmic Wavs Account Action".into(),
        timeout_height: current_height + 100,
        extension_options: vec![],
        non_critical_extension_options: vec![Any {
            type_url: TX_EXTENSION_TYPE.into(),
            value: to_json_binary(&TxExtension { selected_authenticators })?.to_vec(),
        }]
        .to_vec(),
    })
}
