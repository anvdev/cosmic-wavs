use commonware_codec::{DecodeExt, Encode};
use commonware_cryptography::{bls12381::Bls12381Batch, BatchScheme, Bls12381, Signer, Verifier};

use cosmrs::{bip32::secp256k1::elliptic_curve::rand_core::OsRng, tx::MessageExt};
use cosmwasm_std::StdError;

#[test]
fn main() {
    println!("BLS12-381 Signature Demo");
    println!("=======================\n");

    // PART 1: Generate new keys randomly
    println!("1. Generating new keys randomly");
    println!("------------------------------");

    // Generate multiple signers
    let mut signers = vec![];
    let n_signers = 3;

    println!("Generating {} signers...", n_signers);
    for i in 0..n_signers {
        let mut signer = Bls12381::new(&mut OsRng);
        let private_key = signer.private_key();
        let public_key = signer.public_key();

        println!("Signer {}:", i);
        println!("  Private key: {}", hex::encode(private_key.as_ref()));
        println!("  Public key: {}", hex::encode(public_key.to_string()));

        // Store private key bytes for later import example
        if i == 0 {
            println!("\nSaving first private key for later import");
            let encoded_private_key = private_key.encode();
            println!("  Encoded bytes: {}", hex::encode(&encoded_private_key));
        }

        signers.push(signer);
    }

    // PART 2: Import existing private key
    println!("2. Importing existing private keys");
    println!("--------------------------------");

    // Import the private key
    let imported_private_key =
        match <Bls12381 as commonware_cryptography::Signer>::PrivateKey::decode(
            signers[0].private_key().as_ref(), // hex::decode(PRIVKEY.as_bytes()).unwrap().as_ref()
        ) {
            Ok(key) => key,
            Err(e) => {
                println!("Failed to decode private key: {:?}", e);
                return;
            }
        };

    // Create a signer from the imported key
    let imported_signer = <Bls12381 as commonware_cryptography::Signer>::from(imported_private_key)
        .expect("broken private key");

    // Verify the public key matches the original
    println!(
        "Public key from imported private key: {}",
        hex::encode(imported_signer.public_key().to_string())
    );
    println!();

    // PART 3: Sign and verify messages
    println!("3. Signing and verifying messages");
    println!("--------------------------------");

    // Create messages for each signer
    let namespace = Some(&b"demo"[..]);
    let messages = vec![
        b"Message 1: Hello, world!".to_vec(),
        b"Message 2: BLS signatures".to_vec(),
        b"Message 3: can be aggregated".to_vec(),
    ];

    // Sign messages
    println!("Signing messages...");
    let mut public_keys = vec![];
    let mut signatures = vec![];

    for (i, (signer, message)) in signers.iter_mut().zip(messages.iter()).enumerate() {
        let signature = signer.sign(namespace, message);

        println!("Signature {}: {}", i, hex::encode(signature.to_string()));

        // Store public keys and signatures for verification
        public_keys.push(signer.public_key());
        signatures.push(signature);
    }
    println!();

    // Individual verification
    println!("Verifying individual signatures...");
    for (i, (public_key, (message, signature))) in
        public_keys.iter().zip(messages.iter().zip(signatures.iter())).enumerate()
    {
        let is_valid = Bls12381::verify(namespace, message, public_key, signature);
        println!("Signature {}: {}", i, if is_valid { "Valid" } else { "Invalid" });
    }
    println!();

    // PART 4: Batch verification
    println!("4. Performing batch verification");
    println!("-------------------------------");

    let mut batch = Bls12381Batch::new();

    for (public_key, (message, signature)) in
        public_keys.iter().zip(messages.iter().zip(signatures.iter()))
    {
        batch.add(namespace, message, public_key, signature);
    }

    let batch_valid = batch.verify(&mut OsRng);
    println!("Batch verification result: {}", if batch_valid { "Valid" } else { "Invalid" });
    println!();

    // Demonstrate what happens with invalid signature
    println!("Demonstrating batch verification with tampered message...");
    let mut batch = Bls12381Batch::new();

    // Add all signatures to the batch, but tamper with one message
    for (i, (public_key, (message, signature))) in
        public_keys.iter().zip(messages.iter().zip(signatures.iter())).enumerate()
    {
        let message_to_verify = if i == 1 {
            // Tamper with the second message
            b"TAMPERED: this will fail verification"
        } else {
            message.as_slice()
        };

        batch.add(namespace, message_to_verify, public_key, signature);
    }

    let batch_valid = batch.verify(&mut OsRng);
    println!(
        "Batch verification with tampered message: {}",
        if batch_valid { "Valid" } else { "Invalid" }
    );
}
