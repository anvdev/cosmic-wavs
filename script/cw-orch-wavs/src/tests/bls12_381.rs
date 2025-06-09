use commonware_cryptography::{
    bls12381::{
        primitives::{
            group::{Element, Point, Scalar, G1, G2},
            ops::{
                aggregate_signatures, aggregate_verify_multiple_public_keys, keypair, sign_message,
            },
            variant::{MinPk, MinSig, Variant},
        },
        Bls12381Batch, PublicKey, Signature,
    },
    BatchScheme, Bls12381, Signer, Verifier,
};

use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;
use cosmrs::{bip32::secp256k1::elliptic_curve::rand_core::OsRng, Any};
use cosmwasm_std::{testing::mock_dependencies, to_json_binary, Api, HashFunction};
use hex;
use sha2::{Digest, Sha256};

use commonware_codec::{extensions::DecodeExt, Encode};
use rand::thread_rng;

// Function to generate multiple signers and return their private keys
fn generate_keys(n_signers: usize) -> (Vec<Bls12381>, Vec<Vec<u8>>) {
    println!("Generating {} signers...", n_signers);
    let mut signers = vec![];
    let mut private_keys = vec![];

    for i in 0..n_signers {
        let signer = Bls12381::new(&mut OsRng);
        let private_key = signer.private_key();
        let public_key = signer.public_key();

        println!("Signer {}:", i);
        println!("  Private key: {}", hex::encode(private_key.as_ref()));
        println!("  Public key: {}", hex::encode(public_key.to_string()));

        // Save the first private key's encoded bytes for demonstration
        if i == 0 {
            println!("\nSaving first private key for later import");
            let encoded_private_key = private_key.encode();
            println!("  Encoded bytes: {}", hex::encode(&encoded_private_key));
        }

        private_keys.push(private_key.as_ref().to_vec());
        signers.push(signer);
    }
    println!();
    (signers, private_keys)
}

// Function to import a private key and create a signer
fn import_private_key(encoded_key: &[u8]) -> Result<Bls12381, String> {
    println!("Importing private key...");
    let private_key = <Bls12381 as Signer>::PrivateKey::decode(encoded_key)
        .map_err(|e| format!("Failed to decode private key: {:?}", e))?;
    let signer = <Bls12381 as Signer>::from(private_key).expect("broken private key");

    println!(
        "Public key from imported private key: {}",
        hex::encode(signer.public_key().to_string())
    );
    Ok(signer)
}

// Function to sign messages with multiple signers
fn sign_messages(
    signers: &mut [Bls12381],
    messages: &[Vec<u8>],
    namespace: Option<&[u8]>,
) -> Vec<(Vec<u8>, Signature)> {
    println!("Signing messages...");
    let mut signatures = vec![];

    for (i, (signer, message)) in signers.iter_mut().zip(messages.iter()).enumerate() {
        let signature = signer.sign(namespace, message);
        println!("Message {}: {}", i, hex::encode(message));
        println!("Signature {}: {}", i, hex::encode(signature.to_string()));
        signatures.push((message.clone(), signature));
    }
    println!();
    signatures
}

// Function to verify individual signatures
fn verify_individual_signatures(
    public_keys: &[commonware_cryptography::bls12381::PublicKey],
    messages_and_signatures: &[(Vec<u8>, Signature)],
    namespace: Option<&[u8]>,
) {
    println!("Verifying individual signatures...");
    for (i, (public_key, (message, signature))) in
        public_keys.iter().zip(messages_and_signatures.iter()).enumerate()
    {
        let is_valid = Bls12381::verify(namespace, message, public_key, signature);
        println!("Signature {}: {}", i, if is_valid { "Valid" } else { "Invalid" });
    }
    println!();
}

// Function to perform batch verification
fn verify_batch(
    public_keys: &[commonware_cryptography::bls12381::PublicKey],
    messages_and_signatures: &[(Vec<u8>, Signature)],
    namespace: Option<&[u8]>,
    tamper_index: Option<usize>,
) -> bool {
    println!("Performing batch verification...");
    let mut batch = Bls12381Batch::new();

    for (i, (public_key, (message, signature))) in
        public_keys.iter().zip(messages_and_signatures.iter()).enumerate()
    {
        let message_to_verify = if tamper_index == Some(i) {
            println!("Tampering with message {} for demonstration", i);
            b"TAMPERED: this will fail verification".to_vec()
        } else {
            message.clone()
        };
        batch.add(namespace, &message_to_verify, public_key, signature);
    }

    let batch_valid = batch.verify(&mut OsRng);
    println!("Batch verification result: {}", if batch_valid { "Valid" } else { "Invalid" });
    batch_valid
}

// same as cosmwasm-std library
pub const BLS12_381_G1_GENERATOR: [u8; 48] = [
    151, 241, 211, 167, 49, 151, 215, 148, 38, 149, 99, 140, 79, 169, 172, 15, 195, 104, 140, 79,
    151, 116, 185, 5, 161, 78, 58, 63, 23, 27, 172, 88, 108, 85, 232, 63, 249, 122, 26, 239, 251,
    58, 240, 10, 219, 34, 198, 187,
];

#[test]
fn main() {
    println!("BLS12-381 Signature Demo");
    println!("=======================\n");
    println!("{}", hex::encode(BLS12_381_G1_GENERATOR));

    // PART 1: Generate new keys randomly
    println!("1. Generating new keys randomly");
    println!("------------------------------");
    let n_signers = 3;
    let (mut signers, private_keys) = generate_keys(n_signers);

    // PART 2: Import existing private key
    println!("2. Importing existing private keys");
    println!("--------------------------------");
    let _imported_signer =
        import_private_key(&private_keys[0]).expect("Failed to import private key");

    // PART 3: Sign and verify messages
    println!("3. Signing and verifying messages");
    println!("--------------------------------");
    let namespace = Some(&b"demo"[..]);
    let messages = vec![
        b"Message 1: Hello, world!".to_vec(),
        b"Message 2: BLS signatures".to_vec(),
        b"Message 3: can be aggregated".to_vec(),
    ];
    let signatures = sign_messages(&mut signers, &messages, namespace);

    // Extract public keys for verification
    let public_keys: Vec<commonware_cryptography::bls12381::PublicKey> =
        signers.iter().map(|s| s.public_key()).collect();

    // Individual verification
    verify_individual_signatures(&public_keys, &signatures, namespace);

    // PART 4: Batch verification
    println!("4. Performing batch verification");
    println!("-------------------------------");
    verify_batch(&public_keys, &signatures, namespace, None);

    // PART 5: Batch verification with tampered message
    println!("5. Demonstrating batch verification with tampered message");
    println!("-------------------------------------------------------");
    verify_batch(&public_keys, &signatures, namespace, Some(1));

    // PART 6: Verify all keys signed the same message
    println!("6. Verifying all keys signed the same message");
    println!("--------------------------------------------");
    let common_message = b"Common message for all signers".to_vec();
    verify_same_message(&mut signers, &public_keys, &common_message, namespace);

    // PART 7: Verify aggregated signature for the same message
    println!("7. Verifying aggregated signature");
    println!("---------------------------------");
    let agg_message = b"Aggregated message".to_vec();
    // Case 1: All signers sign the same message

    if !verify_aggregated_signature(&mut signers, &public_keys, &agg_message, namespace, None) {
        panic!("successful authorization expected")
    }

    // Case 2: One signer signs a different message
    println!("\n7b. Verifying aggregated signature with one different message");
    println!("---------------------------------------------------------");

    if verify_aggregated_signature(&mut signers, &public_keys, &agg_message, namespace, Some(1)) {
        panic!("unsuccessful authorization expected")
    }

    // Additional verification from commonware tests
    aggregate_verify_wrong_public_keys::<MinPk>();
    aggregate_verify_wrong_public_keys::<MinSig>();

    // Return private keys (for testing or reuse)
    println!("\nGenerated private keys:");
    for (i, key) in private_keys.iter().enumerate() {
        println!("Private key {}: {}", i, hex::encode(key));
    }
}

// New function to verify all keys signed the same message
fn verify_same_message(
    signers: &mut [Bls12381],
    public_keys: &[commonware_cryptography::bls12381::PublicKey],
    message: &[u8],
    namespace: Option<&[u8]>,
) {
    println!("Verifying all keys signed the same message...");
    println!("Message: {}", hex::encode(message));

    for (i, (signer, public_key)) in signers.iter_mut().zip(public_keys.iter()).enumerate() {
        let signature = signer.sign(namespace, message);
        println!("Signature {}: {}", i, hex::encode(signature.to_string()));

        let is_valid = Bls12381::verify(namespace, message, public_key, &signature);
        println!("Signature {}: {}", i, if is_valid { "Valid" } else { "Invalid" });
    }
    println!();
}

// New function to verify aggregated signature for the same message
fn verify_aggregated_signature(
    signers: &mut [Bls12381],
    pks: &[PublicKey],
    message: &[u8],
    namespace: Option<&[u8]>,
    tamper_index: Option<usize>,
) -> bool {
    println!("Verifying aggregated signature for the same message...");
    println!("Message: {}", hex::encode(message));
    let mut res = true;

    // Collect signatures (with optional tampering)
    let mut signatures = vec![];
    for (i, (signer, _public_key)) in signers.iter_mut().zip(pks.iter()).enumerate() {
        let message_to_sign = if tamper_index == Some(i) {
            println!("Tampering msg {}: {}", i, hex::encode(b"TAMPERED: different message"));
            b"TAMPERED: different message".to_vec()
        } else {
            message.to_vec()
        };

        let signature = sign_message::<MinPk>(
            &Scalar::decode(signer.private_key().as_ref())
                .map_err(|_| {
                    res = false;
                })
                .unwrap(),
            namespace,
            &message_to_sign,
        );

        signatures.push(signature);
    }

    // Case 1: Prepare public keys and signatures for aggregation
    // Public keys are already in the correct form (G1 points, since PublicKey = G1)
    let ag1s: Vec<G1> = pks.iter().map(|pk| G1::decode(pk.encode()).expect("agggg")).collect();
    let scalars: Vec<Scalar> = vec![Scalar::one(); ag1s.len()];
    let _agg1pk = G1::msm(ag1s.iter().as_ref(), &scalars);

    // Aggregate signatures on G2 (sum of signatures)
    let signature_scalars: Vec<Scalar> = vec![Scalar::one(); signatures.len()];

    match aggregate_verify_multiple_public_keys::<MinPk, _>(
        ag1s.iter().map(|r| r).collect::<Vec<_>>(),
        namespace,
        message,
        &G2::msm(&signatures, &signature_scalars),
    ) {
        Ok(_) => return res,
        Err(_) => false,
    }
}

fn aggregate_verify_wrong_public_keys<V: Variant>() {
    // Generate signatures
    let (private1, public1) = keypair::<_, V>(&mut thread_rng());
    let (private2, public2) = keypair::<_, V>(&mut thread_rng());
    let (private3, _) = keypair::<_, V>(&mut thread_rng());
    let namespace = b"test";
    let message = b"message";
    let sig1 = sign_message::<V>(&private1, Some(namespace), message);
    let sig2 = sign_message::<V>(&private2, Some(namespace), message);
    let sig3 = sign_message::<V>(&private3, Some(namespace), message);
    let signatures = vec![sig1, sig2, sig3];

    // Aggregate the signatures
    let aggregate_sig = aggregate_signatures::<V, _>(&signatures);

    // Verify the aggregated signature
    let (_, public4) = keypair::<_, V>(&mut thread_rng());
    let wrong_pks = vec![public1, public2, public4];
    let result = aggregate_verify_multiple_public_keys::<V, _>(
        &wrong_pks,
        Some(namespace),
        message,    
        &aggregate_sig,
    );
    assert!(matches!(
        result,
        Err(commonware_cryptography::bls12381::primitives::Error::InvalidSignature)
    ));
}

#[test]
fn test_how_wavs_infusion_service_generates_signature() -> anyhow::Result<()> {
    let mut signatures = vec![];

    // Decode the private key for BLS12-381
    let bls_key_pair = <Bls12381 as commonware_cryptography::Signer>::PrivateKey::decode(
        hex::decode("5ea020a2126f5658bda5c94663a8b5a8e4917d2a7426e0d01b148a67734451b3")?.as_ref(),
    )?;

    // Create a signer from the private key
    let mut imported_signer = <Bls12381 as commonware_cryptography::Signer>::from(bls_key_pair)
        .expect("Failed to create signer");

    // Create a sample Any message for MsgExecuteContract
    let wavs_any_msg = Any::from_msg(&MsgExecuteContract {
        sender: "check12".to_string(), // wavs secp256k1 key address registered with BLS12 authenticator
        contract: "eretskere".to_string(),
        msg: to_json_binary(&cw_infuser::msg::ExecuteMsg::WavsEntryPoint { infusions: vec![] })?
            .to_vec(),
        funds: vec![],
    })?;

    // Compute the SHA-256 digest of the JSON-encoded Any message array
    let msg_digest: [u8; 32] = Sha256::digest(to_json_binary(&vec![wavs_any_msg])?.as_ref())
        .to_vec()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid digest length"))?;

    // Define the domain separation tag for signing
    // let namespace = Some(&b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_"[..]);

    // Generate the signature using the commonware-cryptography signer
    let signature = imported_signer.sign(None, &msg_digest);
    signatures.push(signature.to_vec());

    // Set up mock dependencies for cosmwasm-std

    let deps = mock_dependencies();

    // Define the domain separation tag for verification (must match signing DST)
    // same as commonware-cryptography
    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

    // Aggregate signatures (in this case, only one signature)
    let aggregated_signature = deps
        .api
        .bls12_381_aggregate_g2(&signatures.concat())
        .map_err(|e| anyhow::anyhow!("Signature aggregation failed: {}", e))?;

    // Aggregate public keys (in this case, only one public key)
    let aggregated_pubkey = deps
        .api
        .bls12_381_aggregate_g1(&vec![imported_signer.public_key().to_vec()].concat())
        .map_err(|e| anyhow::anyhow!("Public key aggregation failed: {}", e))?;

    // Hash the message to G2 for verification
    let hashed_message = deps
        .api
        .bls12_381_hash_to_g2(
            HashFunction::Sha256,
            &msg_digest, // Use the raw digest directly, as it's already hashed
            dst,
        )
        .map_err(|e| anyhow::anyhow!("Message hashing to G2 failed: {}", e))?;

    println!("BLS12_381_G1_GENERATOR: {:#?}", hex::encode(BLS12_381_G1_GENERATOR));
    println!("aggregated_signature: {:#?}", hex::encode(aggregated_signature));
    println!("aggregated_pubkey: {:#?}", hex::encode(aggregated_pubkey));
    println!("hashed_message: {:#?}", hex::encode(hashed_message));
    // Verify the signature using pairing equality: e(G1_GENERATOR, aggregated_signature) == e(aggregated_pubkey, hashed_message)
    let verification_result = deps
        .api
        .bls12_381_pairing_equality(
            &BLS12_381_G1_GENERATOR,
            &aggregated_signature,
            &aggregated_pubkey,
            &hashed_message,
        )
        .map_err(|e| anyhow::anyhow!("Pairing equality check failed: {}", e))?;

    if !verification_result {
        return Err(anyhow::anyhow!("Signature verification failed"));
    }

    Ok(())
}
