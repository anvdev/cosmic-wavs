use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub fuel_limit: u64,
    pub max_gas: u64,
    pub file_location: String,
    pub trigger_event: String,
    pub trigger_chain: String,
    pub submit_chain: String,
    pub TRIGGER_ORIGIN: Option<String>,
    pub cosmos_rpc_url: Option<String>,
    pub cosmos_chain_id: Option<String>,
    pub aggregator_url: Option<String>,
    pub deploy_env: String,
    pub wavs_endpoint: String,
    pub registry: String,
    pub pkg_namespace: String,
    pub pkg_name: String,
    pub pkg_version: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            fuel_limit: 1000000000000,
            max_gas: 5000000,
            file_location: ".docker/service.json".to_string(),
            trigger_event: "NewTrigger(bytes)".to_string(),
            trigger_chain: "local".to_string(),
            submit_chain: "local".to_string(),
            TRIGGER_ORIGIN: None,
            cosmos_rpc_url: Some("http://localhost:26657".to_string()),
            cosmos_chain_id: Some("sub-1".to_string()),
            aggregator_url: None,
            deploy_env: "LOCAL".to_string(),
            wavs_endpoint: "http://localhost:8000".to_string(),
            registry: "wa.dev".to_string(),
            pkg_namespace: "wavs".to_string(),
            pkg_name: "component".to_string(),
            pkg_version: "latest".to_string(),
        }
    }
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Load values from environment variables
        if let Ok(val) = env::var("FUEL_LIMIT") {
            config.fuel_limit = val.parse().context("Invalid FUEL_LIMIT")?;
        }
        if let Ok(val) = env::var("MAX_GAS") {
            config.max_gas = val.parse().context("Invalid MAX_GAS")?;
        }
        if let Ok(val) = env::var("FILE_LOCATION") {
            config.file_location = val;
        }
        if let Ok(val) = env::var("TRIGGER_EVENT") {
            config.trigger_event = val;
        }
        if let Ok(val) = env::var("TRIGGER_CHAIN") {
            config.trigger_chain = val;
        }
        if let Ok(val) = env::var("SUBMIT_CHAIN") {
            config.submit_chain = val;
        }
        if let Ok(val) = env::var("TRIGGER_ORIGIN") {
            config.TRIGGER_ORIGIN = Some(val);
        }
        if let Ok(val) = env::var("COSMOS_RPC_URL") {
            config.cosmos_rpc_url = Some(val);
        }
        if let Ok(val) = env::var("COSMOS_CHAIN_ID") {
            config.cosmos_chain_id = Some(val);
        }
        if let Ok(val) = env::var("AGGREGATOR_URL") {
            config.aggregator_url = Some(val);
        }
        if let Ok(val) = env::var("DEPLOY_ENV") {
            config.deploy_env = val;
        }
        if let Ok(val) = env::var("WAVS_ENDPOINT") {
            config.wavs_endpoint = val;
        }
        if let Ok(val) = env::var("REGISTRY") {
            config.registry = val;
        }
        if let Ok(val) = env::var("PKG_NAMESPACE") {
            config.pkg_namespace = val;
        }
        if let Ok(val) = env::var("PKG_NAME") {
            config.pkg_name = val;
        }
        if let Ok(val) = env::var("PKG_VERSION") {
            config.pkg_version = val;
        }

        Ok(config)
    }
}

/// Build WAVS service configuration
pub fn build_service_config(config: ServiceConfig) -> Result<String> {
    // Get required addresses
    let service_manager_address = env::var("SERVICE_MANAGER_ADDRESS")
        .or_else(|_| get_service_manager_from_deploy())
        .context("SERVICE_MANAGER_ADDRESS not found")?;

    let trigger_address = env::var("TRIGGER_ADDRESS")
        .or_else(|_| get_trigger_from_deploy())
        .context("TRIGGER_ADDRESS not found")?;

    let submit_address = env::var("SUBMIT_ADDRESS")
        .or_else(|_| get_submit_from_deploy())
        .context("SUBMIT_ADDRESS not found")?;

    // Create base docker command
    let base_cmd = format!(
        "docker run --rm --network host -w /data -v {}:/data ghcr.io/lay3rlabs/wavs:99aa44a wavs-cli service --json true --home /data --file /data/{}",
        env::current_dir()?.display(),
        config.file_location
    );

    // Initialize service
    let service_id = run_wavs_command(&format!("{} init --name demo", base_cmd))
        .and_then(|output| {
            let json: Value = serde_json::from_str(&output)?;
            Ok(json["service"]["id"].as_str().unwrap_or("").to_string())
        })
        .context("Failed to initialize service")?;

    println!("Service ID: {}", service_id);

    // Add workflow
    let workflow_id = run_wavs_command(&format!("{} workflow add", base_cmd))
        .and_then(|output| {
            let json: Value = serde_json::from_str(&output)?;
            Ok(json["workflow_id"].as_str().unwrap_or("").to_string())
        })
        .context("Failed to add workflow")?;

    println!("Workflow ID: {}", workflow_id);

    // Configure trigger based on destination
    if config.TRIGGER_ORIGIN.as_deref() == Some("COSMOS") {
        println!("Configuring Cosmos trigger...");
        let cosmos_rpc = config.cosmos_rpc_url.as_deref().unwrap_or("http://localhost:26657");
        let cosmos_chain = config.cosmos_chain_id.as_deref().unwrap_or("sub-1");
        
        run_wavs_command(&format!(
            "{} workflow trigger --id {} set-cosmos --rpc-url {} --chain-id {} --event-type {}",
            base_cmd, workflow_id, cosmos_rpc, cosmos_chain, config.trigger_event
        ))
        .context("Failed to set Cosmos trigger")?;
    } else {
        println!("Configuring EVM trigger...");
        let trigger_event_hash = run_cast_command(&format!("cast keccak {}", config.trigger_event))?;
        
        run_wavs_command(&format!(
            "{} workflow trigger --id {} set-evm --address {} --chain-name {} --event-hash {}",
            base_cmd, workflow_id, trigger_address, config.trigger_chain, trigger_event_hash.trim()
        ))
        .context("Failed to set EVM trigger")?;
    }

    // Configure submission
    let sub_cmd = if let Some(aggregator_url) = &config.aggregator_url {
        format!("set-aggregator --url {}", aggregator_url)
    } else {
        "set-evm".to_string()
    };

    run_wavs_command(&format!(
        "{} workflow submit --id {} {} --address {} --chain-name {} --max-gas {}",
        base_cmd, workflow_id, sub_cmd, submit_address, config.submit_chain, config.max_gas
    ))
    .context("Failed to set submission")?;

    // Set component source
    run_wavs_command(&format!(
        "{} workflow component --id {} set-source-registry --domain {} --package {}:{} --version {}",
        base_cmd, workflow_id, config.registry, config.pkg_namespace, config.pkg_name, config.pkg_version
    ))
    .context("Failed to set component source")?;

    // Configure component permissions and limits
    run_wavs_command(&format!(
        "{} workflow component --id {} permissions --http-hosts '*' --file-system true",
        base_cmd, workflow_id
    ))
    .context("Failed to set permissions")?;

    // set time limit
    run_wavs_command(&format!(
        "{} workflow component --id {} time-limit --seconds 30",
        base_cmd, workflow_id
    ))
    .context("Failed to set time limit")?;

    // set secret env variable
    run_wavs_command(&format!(
        "{} workflow component --id {} env --values WAVS_ENV_SOME_SECRET",
        base_cmd, workflow_id
    ))
    .context("Failed to set environment")?;

    // fetch values by keys
    run_wavs_command(&format!(
        "{} workflow component --id {} config --values 'key=value,key2=value2'",
        base_cmd, workflow_id
    ))
    .context("Failed to set config")?;

    // Set service manager
    let checksum_address = run_cast_command(&format!("cast --to-checksum {}", service_manager_address))?;
    run_wavs_command(&format!(
        "{} manager set-evm --chain-name {} --address {}",
        base_cmd, config.submit_chain, checksum_address.trim()
    ))
    .context("Failed to set service manager")?;

    // Validate configuration
    run_wavs_command(&format!("{} validate", base_cmd))
        .context("Service validation failed")?;

    println!("Configuration file created at {}. Watching events from '{}' & submitting to '{}'.",
             config.file_location, config.trigger_chain, config.submit_chain);

    Ok(config.file_location)
}

/// Upload component to WAVS registry
pub fn upload_component(component_filename: &str, wavs_endpoint: &str) -> Result<String> {
    let component_path = format!("./compiled/{}", component_filename);
    
    if !Path::new(&component_path).exists() {
        return Err(anyhow::anyhow!("Component file not found: {}", component_path));
    }

    let output = Command::new("wget")
        .args([
            &format!("--post-file={}", component_path),
            "--header=Content-Type: application/wasm",
            "-O", "-",
            &format!("{}/upload", wavs_endpoint)
        ])
        .output()
        .context("Failed to upload component")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Component upload failed: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    let response = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in upload response")?;
    
    let json: Value = serde_json::from_str(&response)
        .context("Failed to parse upload response")?;
    
    Ok(json["digest"].as_str().unwrap_or("").to_string())
}

/// Deploy WAVS service
pub fn deploy_service(service_url: &str, wavs_endpoint: Option<&str>) -> Result<()> {
    if service_url.is_empty() {
        return Err(anyhow::anyhow!("SERVICE_URL is not set"));
    }

    // Check if WAVS endpoint is reachable
    if let Some(endpoint) = wavs_endpoint {
        let health_url = format!("{}/app", endpoint);
        let status = Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", &health_url])
            .output()
            .context("Failed to check WAVS endpoint")?;

        let status_code = String::from_utf8_lossy(&status.stdout);
        if status_code != "200" {
            return Err(anyhow::anyhow!("WAVS endpoint is not reachable: {}", endpoint));
        }
    }

    // Add a small delay to ensure service is ready
    std::thread::sleep(std::time::Duration::from_secs(2));

    let mut cmd = Command::new("docker");
    cmd.args([
        "run", "--rm", "--network", "host",
        "--env-file", ".env",
        "-v", &format!("{}:/data", env::current_dir()?.display()),
        "ghcr.io/lay3rlabs/wavs:99aa44a",
        "wavs-cli", "deploy-service",
        &format!("--service-url={}", service_url),
        "--log-level=debug",
        "--data=/data/.docker",
        "--home=/data"
    ]);

    if let Some(endpoint) = wavs_endpoint {
        cmd.arg(format!("--wavs-endpoint={}", endpoint));
    }

    let output = cmd.output()
        .context("Failed to deploy service")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Service deployment failed: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    println!("Service deployed successfully!");
    Ok(())
}

// Helper functions

fn run_wavs_command(command: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .context("Failed to run WAVS command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("WAVS command failed: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn run_cast_command(command: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .context("Failed to run cast command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Cast command failed: {}", 
                                   String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn get_service_manager_from_deploy() -> Result<String> {
    let path = "./.nodes/avs_deploy.json";
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("AVS deploy file not found: {}", path));
    }

    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    Ok(json["addresses"]["WavsServiceManager"]
        .as_str()
        .unwrap_or("")
        .to_string())
}

fn get_trigger_from_deploy() -> Result<String> {
    let path = "./.docker/trigger.json";
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("Trigger deploy file not found: {}", path));
    }

    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    Ok(json["deployedTo"].as_str().unwrap_or("").to_string())
}

fn get_submit_from_deploy() -> Result<String> {
    let path = "./.docker/submit.json";
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("Submit deploy file not found: {}", path));
    }

    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    Ok(json["deployedTo"].as_str().unwrap_or("").to_string())
}