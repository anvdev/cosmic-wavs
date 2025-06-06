use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

/// Create a new deployer wallet and configure environment
pub fn create_deployer(deploy_env: &str, rpc_url: &str) -> Result<WalletInfo> {
    // Ensure .docker directory exists
    fs::create_dir_all(".docker")?;

    // Generate new mnemonic and wallet
    let output = Command::new("cast")
        .args(["wallet", "new-mnemonic", "--json"])
        .output()
        .context("Failed to generate new wallet")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to create wallet: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    let wallet_json = String::from_utf8(output.stdout)?;
    let wallet_data: Value = serde_json::from_str(&wallet_json)?;

    let private_key = wallet_data["accounts"][0]["private_key"]
        .as_str()
        .context("Failed to get private key")?
        .to_string();

    let mnemonic = wallet_data["mnemonic"]
        .as_str()
        .context("Failed to get mnemonic")?
        .to_string();

    // Get address from private key
    let address_output = Command::new("cast")
        .args(["wallet", "address", &private_key])
        .output()
        .context("Failed to get wallet address")?;

    let address = String::from_utf8(address_output.stdout)?
        .trim()
        .to_string();

    // Save deployer info
    let deployer_info = json!({
        "address": address,
        "private_key": private_key,
        "mnemonic": mnemonic
    });

    let mut file = File::create(".docker/deployer.json")?;
    serde_json::to_writer_pretty(&mut file, &deployer_info)?;

    // Update .env file
    update_env_file("FUNDED_KEY", &private_key)?;

    // Fund wallet if in local environment
    if deploy_env == "LOCAL" {
        fund_wallet_local(&address, rpc_url)?;
        let balance = get_wallet_balance(&address, rpc_url)?;
        println!("Local deployer `{}` funded with {}ether", address, balance);
    } else {
        println!("Fund deployer {} with some ETH, or change this value in the .env", address);
        wait_for_funding(&address, rpc_url)?;
    }

    Ok(WalletInfo {
        address,
        private_key,
        mnemonic,
    })
}

/// Create a new aggregator wallet
pub fn create_aggregator(index: u32, deploy_env: &str, rpc_url: &str) -> Result<WalletInfo> {
    // Ensure directory exists
    let agg_dir = format!("infra/aggregator-{}", index);
    fs::create_dir_all(&agg_dir)?;

    // Generate new wallet
    let output = Command::new("cast")
        .args(["wallet", "new-mnemonic", "--json"])
        .output()
        .context("Failed to generate aggregator wallet")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to create aggregator wallet: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    let wallet_json = String::from_utf8(output.stdout)?;
    let wallet_data: Value = serde_json::from_str(&wallet_json)?;

    let private_key = wallet_data["accounts"][0]["private_key"]
        .as_str()
        .context("Failed to get aggregator private key")?
        .to_string();

    let mnemonic = wallet_data["mnemonic"]
        .as_str()
        .context("Failed to get aggregator mnemonic")?
        .to_string();

    // Get address
    let address_output = Command::new("cast")
        .args(["wallet", "address", &private_key])
        .output()
        .context("Failed to get aggregator address")?;

    let address = String::from_utf8(address_output.stdout)?
        .trim()
        .to_string();

    // Create .env file for aggregator
    let env_content = format!(
        r#"WAVS_AGGREGATOR_CREDENTIAL="{}"
# Mnemonic: {}
"#,
        private_key, mnemonic
    );

    let env_path = format!("{}/{}", agg_dir, ".env");
    fs::write(&env_path, env_content)?;

    // Create start script
    let start_script = format!(
        r#"#!/bin/bash
cd $(dirname "$0") || exit 1

IMAGE=ghcr.io/lay3rlabs/wavs:99aa44a
INSTANCE=wavs-aggregator-{}

docker kill ${{INSTANCE}} > /dev/null 2>&1 || true
docker rm ${{INSTANCE}} > /dev/null 2>&1 || true

docker run -d --name ${{INSTANCE}} --network host -p 8001:8001 --stop-signal SIGKILL --env-file .env --user 1000:1000 -v .:/wavs \
  ${{IMAGE}} wavs-aggregator --log-level debug --host 0.0.0.0 --port 8001
"#,
        index
    );

    let start_path = format!("{}/start.sh", agg_dir);
    fs::write(&start_path, start_script)?;

    // Make start script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&start_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&start_path, perms)?;
    }

    // Copy wavs.toml
    if Path::new("wavs.toml").exists() {
        fs::copy("wavs.toml", format!("{}/wavs.toml", agg_dir))?;
    }

    // Fund wallet if in local environment
    if deploy_env == "LOCAL" {
        fund_wallet_local(&address, rpc_url)?;
        let balance = get_wallet_balance(&address, rpc_url)?;
        println!("Local aggregator `{}` funded with {}ether", address, balance);
    } else {
        println!("Fund aggregator {} with some ETH, or change this value in {}", address, env_path);
        wait_for_funding(&address, rpc_url)?;
    }

    Ok(WalletInfo {
        address,
        private_key,
        mnemonic,
    })
}

/// Update a key in the .env file
fn update_env_file(key: &str, value: &str) -> Result<()> {
    // Ensure .env exists
    if !Path::new(".env").exists() {
        if Path::new(".env.example").exists() {
            fs::copy(".env.example", ".env")?;
        } else {
            return Err(anyhow::anyhow!(".env file not found"));
        }
    }

    let env_content = fs::read_to_string(".env")?;
    let mut lines: Vec<String> = env_content.lines().map(String::from).collect();

    let key_pattern = format!("{}=", key);
    let new_line = format!("{}={}", key, value);

    // Find and replace or append
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&key_pattern) {
            *line = new_line.clone();
            found = true;
            break;
        }
    }

    if !found {
        lines.push(new_line);
    }

    let updated_content = lines.join("\n");
    fs::write(".env", updated_content)?;

    Ok(())
}

/// Fund a wallet in local environment
fn fund_wallet_local(address: &str, rpc_url: &str) -> Result<()> {
    let output = Command::new("cast")
        .args([
            "rpc", "anvil_setBalance", address, "15000000000000000000",
            "--rpc-url", rpc_url
        ])
        .output()
        .context("Failed to fund wallet")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to fund wallet: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

/// Get wallet balance in ether
fn get_wallet_balance(address: &str, rpc_url: &str) -> Result<String> {
    let output = Command::new("cast")
        .args(["balance", "--ether", address, &format!("--rpc-url={}", rpc_url)])
        .output()
        .context("Failed to get wallet balance")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get balance: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Wait for wallet to be funded externally
fn wait_for_funding(address: &str, rpc_url: &str) -> Result<()> {
    use std::{thread, time::Duration};

    loop {
        thread::sleep(Duration::from_secs(5));
        
        let balance = get_wallet_balance(address, rpc_url)?;
        if balance != "0.000000000000000000" {
            println!("Account balance is now {}", balance);
            break;
        }
        println!("      [!] Waiting for balance to be funded by another account...");
    }

    Ok(())
}