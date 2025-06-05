use anyhow::{Context, Result};
use chrono::Utc;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::time::sleep;
use super::{ServiceConfig, build_service_config, deploy_service};

/// Deploy Cosmos WAVS service with integrated Rust workflow
pub async fn deploy_cosmos_service(
    component_filename: &str,
    cosmos_rpc_url: &str,
    cosmos_chain_id: &str,
    trigger_event: &str,
    start_service: bool,
) -> Result<()> {
    // Validate environment and component
    validate_cosmos_deployment(component_filename, cosmos_rpc_url)?;
    
    println!("Deploying Cosmos WAVS service...");
    println!("  Component: {}", component_filename);
    println!("  Cosmos RPC: {}", cosmos_rpc_url);
    println!("  Chain ID: {}", cosmos_chain_id);
    println!("  Trigger Event: {}", trigger_event);

    // 1. Deploy Cosmos contracts using cw-orch-wavs
    deploy_cosmos_contracts().await?;
    
    // 2. Configure environment for Cosmos trigger
    setup_cosmos_environment_vars(cosmos_rpc_url, cosmos_chain_id, trigger_event)?;
    
    // 3. Build service configuration using Rust tools
    let config_file = build_cosmos_service_config().await?;
    
    // 4. Optionally start the service
    if start_service {
        start_cosmos_wavs_service(&config_file).await?;
        println!("Cosmos WAVS service deployed and started!");
    } else {
        println!("Cosmos WAVS service configured at: {}", config_file);
    }

    Ok(())
}

/// Start the Cosmos WAVS service
pub async fn start_cosmos_wavs_service(config_file: &str) -> Result<()> {
    println!("Starting Cosmos WAVS service...");
    
    // Upload service config to IPFS
    let service_hash = upload_to_ipfs(config_file).await?;
    let service_url = format!("ipfs://{}", service_hash);
    
    println!("Service config uploaded to IPFS: {}", service_hash);
    
    // Deploy service using existing tools
    deploy_service(&service_url, Some("http://localhost:8000"))?;
    
    println!("Service deployed successfully!");
    Ok(())
}

/// Upload file to IPFS
pub async fn upload_to_ipfs(file_path: &str) -> Result<String> {
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    let deploy_status = get_deploy_status()?;
    
    let hash = if deploy_status == "LOCAL" {
        // Use local IPFS
        let output = Command::new("curl")
            .args([
                "-X", "POST",
                "http://127.0.0.1:5001/api/v0/add?pin=true",
                "-H", "Content-Type: multipart/form-data",
                "-F", &format!("file=@{}", file_path)
            ])
            .output()
            .context("Failed to upload to local IPFS")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("IPFS upload failed: {}", 
                                       String::from_utf8_lossy(&output.stderr)));
        }

        let response = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&response)?;
        json["Hash"].as_str().unwrap_or("").to_string()
    } else {
        // Use Pinata
        let api_key = env::var("PINATA_API_KEY")
            .context("PINATA_API_KEY is not set for non-local deployment")?;

        let date = Utc::now().format("%b-%d-%Y").to_string();
        let name = format!("service-{}.json", date);

        let output = Command::new("curl")
            .args([
                "-X", "POST",
                "--url", "https://uploads.pinata.cloud/v3/files",
                "--header", &format!("Authorization: Bearer {}", api_key),
                "--header", "Content-Type: multipart/form-data",
                "--form", &format!("file=@{}", file_path),
                "--form", "network=public",
                "--form", &format!("name={}", name)
            ])
            .output()
            .context("Failed to upload to Pinata")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Pinata upload failed: {}", 
                                       String::from_utf8_lossy(&output.stderr)));
        }

        let response = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&response)?;
        json["data"]["cid"].as_str().unwrap_or("").to_string()
    };

    if hash.is_empty() {
        return Err(anyhow::anyhow!("Failed to get IPFS hash from response"));
    }

    Ok(hash)
}

/// Start all local services (Anvil, Docker Compose, etc.)
pub async fn start_all_local(fork_rpc_url: Option<&str>) -> Result<()> {
    let deploy_env = get_deploy_status()?;
    
    if deploy_env == "TESTNET" {
        println!("Running in testnet mode, nothing to do");
        return Ok(());
    }

    if deploy_env == "LOCAL" {
        let rpc_url = fork_rpc_url.unwrap_or("https://ethereum-holesky-rpc.publicnode.com");
        let port = "8545";

        println!("Starting Anvil with fork URL: {}", rpc_url);
        
        // Start Anvil
        let _anvil = Command::new("anvil")
            .args(["--fork-url", rpc_url, "--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start Anvil")?;

        // Wait for Anvil to be ready
        wait_for_rpc(&format!("http://localhost:{}", port)).await?;

        // Check if cosmos trigger is being deployed
        let trigger_dest = env::var("TRIGGER_ORIGIN").unwrap_or_default();
        if trigger_dest == "COSMOS" {
            setup_cosmos_environment().await?;
        }

        // Start Docker Compose services
        println!("Starting Docker Compose services...");
        let output = Command::new("docker")
            .args([
                "compose",
                "-f", "../../docker-compose.yml",
                "-f", "../../telemetry/docker-compose.yml",
                "up", "--force-recreate", "-d"
            ])
            .output()
            .context("Failed to start Docker Compose")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Docker Compose failed: {}", 
                                       String::from_utf8_lossy(&output.stderr)));
        }

        // If Cosmos trigger is enabled, deploy service after containers start
        if trigger_dest == "COSMOS" {
            println!("Cosmos trigger detected - setting up local node...");
            setup_local_cosmos_node().await?;
        }

        println!("Started...");
    }

    Ok(())
}

/// Setup Cosmos environment for local deployment
async fn setup_cosmos_environment() -> Result<()> {
    println!("Setting up Cosmos environment...");
    
    // Load cosmos environment variables from template if available
    load_cosmos_env_template()?;
    
    // Ensure cosmos data directory exists (only for LOCAL)
    let deploy_status = get_deploy_status()?;
    if deploy_status == "LOCAL" {
        fs::create_dir_all("./.cosmos")?;
    }
    
    // Set default environment variables for cosmos development
    set_default_cosmos_env_vars();
    
    println!("Cosmos environment configured");
    Ok(())
}

/// Wait for RPC endpoint to be ready
async fn wait_for_rpc(rpc_url: &str) -> Result<()> {
    for _ in 0..60 { // 30 seconds timeout
        if let Ok(output) = Command::new("cast")
            .args(["block-number", "--rpc-url", rpc_url])
            .output()
        {
            if output.status.success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    
    Err(anyhow::anyhow!("RPC endpoint {} not ready after timeout", rpc_url))
}

/// Wait for Cosmos node to be ready
async fn wait_for_cosmos_node() -> Result<()> {
    let health_url = "http://localhost:26657/health";
    
    for _ in 0..60 { // 5 minutes timeout  
        if let Ok(output) = Command::new("curl")
            .args(["-s", health_url])
            .output()
        {
            if output.status.success() {
                return Ok(());
            }
        }
        println!("Waiting for Cosmos node...");
        sleep(Duration::from_secs(5)).await;
    }
    
    Err(anyhow::anyhow!("Cosmos node not ready after timeout"))
}

/// Get deployment status from environment
fn get_deploy_status() -> Result<String> {
    // Check if .env exists
    if !Path::new(".env").exists() {
        if Path::new(".env.example").exists() {
            fs::copy(".env.example", ".env")?;
        } else {
            return Ok("LOCAL".to_string());
        }
    }

    // Read TRIGGER_ORIGIN from .env
    let env_content = fs::read_to_string(".env")?;
    for line in env_content.lines() {
        if let Some(value) = line.strip_prefix("TRIGGER_ORIGIN=") {
            return Ok(value.to_uppercase());
        }
    }

    Ok("LOCAL".to_string())
}

// New helper functions for clean deployment workflow

/// Validate cosmos deployment prerequisites
fn validate_cosmos_deployment(component_filename: &str, _cosmos_rpc_url: &str) -> Result<()> {
    // Load cosmos environment variables from template if available
    load_cosmos_env_template()?;
    
    // Set default environment variables first
    set_default_cosmos_env_vars();
    
    // Check if component exists
    let component_path = format!("./compiled/{}", component_filename);
    if !Path::new(&component_path).exists() {
        return Err(anyhow::anyhow!("Component file not found: {}", component_path));
    }

    // Validate complete cosmos environment
    validate_cosmos_environment()?;

    println!("✓ Component file exists: {}", component_path);
    println!("✓ Cosmos deployment environment validated");
    
    Ok(())
}

/// Deploy Cosmos contracts using cw-orch framework
async fn deploy_cosmos_contracts() -> Result<()> {
    println!("Deploying Cosmos contracts with cw-orch...");
    
    let deploy_status = get_deploy_status()?;
    let current_dir = env::current_dir()?;
    let cw_orch_dir = current_dir.join("script/cw-orch-wavs");
    
    // Determine network type based on deployment environment
    let (network, use_docker) = match deploy_status.as_str() {
        "LOCAL" => ("local", true),
        "TESTNET" => ("testnet", false),
        "MAINNET" | "MAIN" => ("main", false),
        _ => ("local", true), // Default to local
    };
    
    println!("  Network: {} (docker: {})", network, use_docker);
    
    let mut args = vec!["run", "--bin", "wavs", "deploy", "--network", network];
    
    // Only add docker-compose for local deployments
    if use_docker {
        args.extend_from_slice(&["--docker-compose", "../../docker-compose.yml"]);
        println!("  Using local Docker Compose for Cosmos node deployment");
    } else {
        println!("  Using cw-orch client library for {} network", network.to_uppercase());
    }
    
    let output = Command::new("cargo")
        .args(&args)
        .current_dir(&cw_orch_dir)
        .output()
        .context("Failed to deploy Cosmos contracts")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Cosmos contract deployment failed: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }
    
    println!("✓ Cosmos contracts deployed successfully on {} network", network.to_uppercase());
    Ok(())
}

/// Configure environment variables for Cosmos trigger
fn setup_cosmos_environment_vars(cosmos_rpc_url: &str, cosmos_chain_id: &str, trigger_event: &str) -> Result<()> {
    println!("Configuring Cosmos environment variables...");
    
    env::set_var("TRIGGER_ORIGIN", "COSMOS");
    env::set_var("TRIGGER_CHAIN", "cosmos");
    env::set_var("SUBMIT_CHAIN", "local");
    env::set_var("TRIGGER_EVENT", trigger_event);
    env::set_var("COSMOS_RPC_URL", cosmos_rpc_url);
    env::set_var("COSMOS_CHAIN_ID", cosmos_chain_id);
    env::set_var("FILE_LOCATION", ".docker/cosmos-service.json");
    
    println!("✓ Environment configured for Cosmos trigger");
    Ok(())
}

/// Build service configuration for Cosmos deployment
async fn build_cosmos_service_config() -> Result<String> {
    println!("Building Cosmos service configuration...");
    
    let config = ServiceConfig::from_env()
        .context("Failed to load service config from environment")?;
    
    let config_file = build_service_config(config)
        .context("Failed to build service configuration")?;
    
    println!("✓ Service configuration built: {}", config_file);
    Ok(config_file)
}

/// Setup and deploy local Cosmos node with WAVS integration
async fn setup_local_cosmos_node() -> Result<()> {
    println!("Setting up local Cosmos node with WAVS...");
    
    // Wait for Cosmos node to be ready
    wait_for_cosmos_node().await?;
    
    // Deploy WAVS service with Cosmos configuration
    let component = env::var("COMPONENT_FILENAME")
        .unwrap_or_else(|_| "cosmic-wavs-demo-infusion.wasm".to_string());
    let rpc_url = env::var("COSMOS_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:26657".to_string());
    let chain_id = env::var("COSMOS_CHAIN_ID")
        .unwrap_or_else(|_| "sub-1".to_string());
    let trigger_event = env::var("TRIGGER_EVENT")
        .unwrap_or_else(|_| "cw-infusion".to_string());
    
    deploy_cosmos_service(&component, &rpc_url, &chain_id, &trigger_event, false).await?;
    println!("Local Cosmos WAVS service configured successfully!");
    
    Ok(())
}

/// Load cosmos environment variables from template if available
fn load_cosmos_env_template() -> Result<()> {
    let template_path = "script/template/.env.example.cosmos";
    
    if !Path::new(template_path).exists() {
        // Template not found, skip loading
        return Ok(());
    }
    
    println!("Loading cosmos environment from template: {}", template_path);
    
    let template_content = fs::read_to_string(template_path)
        .context("Failed to read cosmos template file")?;
    
    for line in template_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some(eq_pos) = line.find('=') {
            let key = &line[..eq_pos];
            let value = &line[eq_pos + 1..];
            
            // Only set if not already set in environment
            if env::var(key).is_err() && !value.is_empty() {
                env::set_var(key, value);
                println!("  Set {} from template", key);
            }
        }
    }
    
    Ok(())
}

/// Set complete environment variables for Cosmos development
fn set_default_cosmos_env_vars() {
    println!("Setting default Cosmos environment variables...");
    
    // Core Cosmos network settings (matching docker-compose and entrypoint)
    if env::var("COSMOS_RPC_URL").is_err() {
        env::set_var("COSMOS_RPC_URL", "http://localhost:26657");
    }
    if env::var("COSMOS_CHAIN_ID").is_err() {
        env::set_var("COSMOS_CHAIN_ID", "sub-1");
    }
    if env::var("COSMOS_GRPC_URL").is_err() {
        env::set_var("COSMOS_GRPC_URL", "http://localhost:9090");
    }
    
    // WAVS Controller settings
    if env::var("WAVS_CONTROLLER_ADDRESS").is_err() {
        env::set_var("WAVS_CONTROLLER_ADDRESS", "bitsong1phaxpevm5wecex2jyaqty2a4v02qj7qmlmzk5a");
    }
    if env::var("WAVS_CONTROLLER_MNEMONIC").is_err() {
        env::set_var("WAVS_CONTROLLER_MNEMONIC", 
                     "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius");
    }
    if env::var("WAVS_BLS12_PRIVKEY").is_err() {
        env::set_var("WAVS_BLS12_PRIVKEY", "test-bls-key-for-development");
    }
    if env::var("WAVS_TOML_PATH").is_err() {
        env::set_var("WAVS_TOML_PATH", "./wavs.toml");
    }
    
    // Deployment settings
    if env::var("DEPLOY_ENV").is_err() {
        env::set_var("DEPLOY_ENV", "LOCAL");
    }
    if env::var("TRIGGER_ORIGIN").is_err() {
        env::set_var("TRIGGER_ORIGIN", "COSMOS");
    }
    
    // Component and service settings
    if env::var("COMPONENT_FILENAME").is_err() {
        env::set_var("COMPONENT_FILENAME", "cosmic-wavs-demo-infusion.wasm");
    }
    if env::var("TRIGGER_EVENT").is_err() {
        env::set_var("TRIGGER_EVENT", "cw-infusion");
    }
    
    // Registry and package settings
    if env::var("REGISTRY").is_err() {
        env::set_var("REGISTRY", "wa.dev");
    }
    if env::var("PKG_NAMESPACE").is_err() {
        env::set_var("PKG_NAMESPACE", "wavs");
    }
    if env::var("PKG_NAME").is_err() {
        env::set_var("PKG_NAME", "component");
    }
    if env::var("PKG_VERSION").is_err() {
        env::set_var("PKG_VERSION", "latest");
    }
    
    // WAVS service settings
    if env::var("WAVS_ENDPOINT").is_err() {
        env::set_var("WAVS_ENDPOINT", "http://localhost:8000");
    }
    
    println!("✓ All Cosmos environment variables configured");
}

/// Validate complete cosmos deployment environment
fn validate_cosmos_environment() -> Result<()> {
    println!("Validating Cosmos deployment environment...");
    
    // Required Cosmos variables
    let required_vars = [
        ("COSMOS_RPC_URL", "Cosmos RPC endpoint"),
        ("COSMOS_CHAIN_ID", "Cosmos chain ID"),
        ("WAVS_CONTROLLER_ADDRESS", "WAVS controller address"),
        ("WAVS_CONTROLLER_MNEMONIC", "WAVS controller mnemonic"),
        ("WAVS_BLS12_PRIVKEY", "WAVS BLS12 private key"),
        ("WAVS_TOML_PATH", "WAVS configuration path"),
        ("COMPONENT_FILENAME", "Component filename"),
        ("TRIGGER_EVENT", "Trigger event name"),
    ];
    
    let mut missing_vars = Vec::new();
    for (var_name, description) in required_vars.iter() {
        if env::var(var_name).is_err() {
            missing_vars.push(format!("{} ({})", var_name, description));
        }
    }
    
    if !missing_vars.is_empty() {
        return Err(anyhow::anyhow!(
            "Missing required environment variables: {}", 
            missing_vars.join(", ")
        ));
    }
    
    let deploy_status = get_deploy_status()?;
    
    // Only validate/create cosmos node directories for LOCAL deployments
    if deploy_status == "LOCAL" {
        // Validate cosmos node directories (matching docker-compose volume mounts)
        if !Path::new("./.cosmos").exists() {
            fs::create_dir_all("./.cosmos")?;
            println!("✓ Created .cosmos directory (for local cosmos node data)");
        }
        
        // Ensure cosmos test-keys directory exists for entrypoint script
        if !Path::new("./.cosmos/test-keys").exists() {
            fs::create_dir_all("./.cosmos/test-keys")?;
            println!("✓ Created .cosmos/test-keys directory");
        }
        
        println!("✓ Local cosmos node directories validated");
    } else {
        println!("✓ Skipping local directories for {} deployment (using cw-orch client)", deploy_status);
    }
    
    // Always ensure .docker directory exists for service configs
    if !Path::new("./.docker").exists() {
        fs::create_dir_all("./.docker")?;
        println!("✓ Created .docker directory (for service configs)");
    }
    
    println!("✓ Cosmos environment validation passed");
    Ok(())
}