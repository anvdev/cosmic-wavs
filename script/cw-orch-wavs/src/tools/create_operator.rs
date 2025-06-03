use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub fn create_operator(index: Option<u32>, force: bool) -> Result<()> {
    // Get operator index from argument or environment variable
    let operator_index = match index {
        Some(idx) => idx,
        None => std::env::var("OPERATOR_INDEX")
            .context("Please provide an operator index as argument or set OPERATOR_INDEX environment variable")?
            .parse::<u32>()
            .context("OPERATOR_INDEX must be a valid number")?,
    };

    // Find git root directory
    let git_root = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to find git root directory")?;

    if !git_root.status.success() {
        anyhow::bail!("Not in a git repository");
    }

    let git_root_path = String::from_utf8(git_root.stdout)?.trim().to_string();
    std::env::set_current_dir(&git_root_path).context("Failed to change to git root directory")?;

    // Create .docker directory
    fs::create_dir_all(".docker").context("Failed to create .docker directory")?;

    let operator_loc = format!("infra/wavs-{}", operator_index);
    let operator_path = Path::new(&operator_loc);

    // Check if directory exists and handle accordingly
    if operator_path.exists() && operator_path.read_dir()?.next().is_some() {
        if !force {
            print!(
                "Directory {} already exists and is not empty. Do you want to remove it? (y/n): ",
                operator_loc
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Exiting without changes.");
                return Ok(());
            }
        }

        println!("Removing {}", operator_loc);

        // Kill existing docker container
        let _ =
            Command::new("docker").args(&["kill", &format!("wavs-{}", operator_index)]).output();

        println!("Removing dir {} (may prompt for password)", operator_loc);

        // Remove directory (using sudo if needed)
        let remove_result = Command::new("rm").args(&["-rf", &operator_loc]).status();

        if remove_result.is_err() || !remove_result.unwrap().success() {
            Command::new("sudo")
                .args(&["rm", "-rf", &operator_loc])
                .status()
                .context("Failed to remove existing operator directory")?;
        }
    }

    // Create operator directory
    fs::create_dir_all(&operator_loc).context("Failed to create operator directory")?;

    // Create .env file
    let env_filename = format!("{}/.env", operator_loc);
    let template_path = "./script/template/.env.example.operator";

    if Path::new(template_path).exists() {
        fs::copy(template_path, &env_filename).context("Failed to copy .env template")?;
    } else {
        // Create a basic .env template if the original doesn't exist
        let env_content = r#"# WAVS Operator Environment Configuration
WAVS_SUBMISSION_MNEMONIC=""
WAVS_CLI_EVM_CREDENTIAL=""
"#;
        fs::write(&env_filename, env_content).context("Failed to create .env file")?;
    }

    // Generate new wallet using cast
    let temp_filename = ".docker/tmp.json";
    let wallet_output =
        Command::new("cast").args(&["wallet", "new-mnemonic", "--json"]).output().context(
            "Failed to generate new wallet. Make sure 'cast' is installed and available in PATH",
        )?;

    if !wallet_output.status.success() {
        anyhow::bail!(
            "Failed to generate wallet: {}",
            String::from_utf8_lossy(&wallet_output.stderr)
        );
    }

    fs::write(temp_filename, &wallet_output.stdout)
        .context("Failed to write temporary wallet file")?;

    // Parse wallet JSON
    let wallet_json: serde_json::Value =
        serde_json::from_slice(&wallet_output.stdout).context("Failed to parse wallet JSON")?;

    let mnemonic =
        wallet_json["mnemonic"].as_str().context("Failed to extract mnemonic from wallet")?;

    let private_key = wallet_json["accounts"][0]["private_key"]
        .as_str()
        .context("Failed to extract private key from wallet")?;

    // Update .env file with wallet credentials
    let env_content = fs::read_to_string(&env_filename).context("Failed to read .env file")?;

    let updated_env = env_content
        .lines()
        .map(|line| {
            if line.starts_with("WAVS_SUBMISSION_MNEMONIC=") {
                format!("WAVS_SUBMISSION_MNEMONIC=\"{}\"", mnemonic)
            } else if line.starts_with("WAVS_CLI_EVM_CREDENTIAL=") {
                format!("WAVS_CLI_EVM_CREDENTIAL=\"{}\"", private_key)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(&env_filename, updated_env).context("Failed to update .env file")?;

    // Clean up temporary file
    let _ = fs::remove_file(temp_filename);

    // Create startup script
    let startup_script = format!(
        r#"#!/bin/bash
cd $(dirname "$0") || exit 1

IMAGE=ghcr.io/lay3rlabs/wavs:99aa44a
WAVS_INSTANCE=wavs-{}

docker kill ${{WAVS_INSTANCE}} > /dev/null 2>&1 || true
docker rm ${{WAVS_INSTANCE}} > /dev/null 2>&1 || true

docker run -d --rm --name ${{WAVS_INSTANCE}} --network host --env-file .env -v $(pwd):/root/wavs ${{IMAGE}} wavs --home /root/wavs --host 0.0.0.0 --log-level info
sleep 0.25

if [ ! "$(docker ps -q -f name=${{WAVS_INSTANCE}})" ]; then
  echo "Container ${{WAVS_INSTANCE}} is not running. Reason:"
  docker run --rm --name ${{WAVS_INSTANCE}} --network host --env-file .env -v $(pwd):/root/wavs ${{IMAGE}} wavs --home /root/wavs --host 0.0.0.0 --log-level info
fi
"#,
        operator_index
    );

    let start_script_path = format!("{}/start.sh", operator_loc);
    fs::write(&start_script_path, startup_script).context("Failed to create startup script")?;

    // Make startup script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&start_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&start_script_path, perms)?;
    }

    // Copy wavs.toml if it exists
    if Path::new("wavs.toml").exists() {
        fs::copy("wavs.toml", format!("{}/wavs.toml", operator_loc))
            .context("Failed to copy wavs.toml")?;
    }

    println!("Operator {} created at {}", operator_index, operator_loc);
    Ok(())
}
