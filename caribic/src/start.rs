use crate::check::check_osmosisd;
use crate::logger::{log_or_show_progress, verbose, warn};
use crate::setup::{
    configure_local_cardano_devnet, copy_cardano_env_file, prepare_db_sync, seed_cardano_devnet,
};
use crate::utils::{
    execute_script, execute_script_with_progress, extract_tendermint_client_id,
    extract_tendermint_connection_id, wait_for_health_check, wait_until_file_exists,
};
use crate::{
    config,
    logger::{self, error, log},
};
use console::style;
use dirs::home_dir;
use fs_extra::copy_items;
use fs_extra::file::copy;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::fs::remove_dir_all;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub fn start_relayer(relayer_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let options = fs_extra::file::CopyOptions::new().overwrite(true);
    copy(
        relayer_path.join(".env.example"),
        relayer_path.join(".env"),
        &options,
    )
    .map_err(|error| format!("Error copying template .env file {}", error.to_string()))?;
    execute_script(relayer_path, "docker", Vec::from(["compose", "stop"]), None)?;

    execute_script_with_progress(
        relayer_path,
        "docker",
        Vec::from(["compose", "up", "-d", "--build"]),
        "⚡ Starting relayer...",
    )?;
    Ok(())
}

pub async fn start_local_cardano_network(
    project_root_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let optional_progress_bar = match logger::get_verbosity() {
        logger::Verbosity::Verbose => None,
        _ => Some(ProgressBar::new_spinner()),
    };

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::with_template("{prefix:.bold} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        progress_bar.set_prefix("🏗 Creating local Cardano network ...".to_owned());
    } else {
        log("🏗 Creating local Cardano network ...");
    }

    let cardano_dir = project_root_path.join("chains/cardano");
    log_or_show_progress(
        &format!(
            "{} 🛠️ Configuring local Cardano devnet",
            style("Step 1/5").bold().dim(),
        ),
        &optional_progress_bar,
    );
    configure_local_cardano_devnet(cardano_dir.as_path())?;
    log_or_show_progress(
        &format!(
            "{} 📝 Copying Cardano environment file",
            style("Step 2/5").bold().dim(),
        ),
        &optional_progress_bar,
    );
    copy_cardano_env_file(project_root_path.join("cardano").as_path())?;
    log_or_show_progress(
        &format!(
            "{} 🛠️ Building Aiken validators",
            style("Step 3/5").bold().dim()
        ),
        &optional_progress_bar,
    );
    execute_script(
        project_root_path.join("cardano").as_path(),
        "aiken",
        Vec::from(["build", "--trace-level", "verbose"]),
        None,
    )?;
    log_or_show_progress(
        &format!(
            "{} 🤖 Generating validator off-chain types",
            style("Step 4/5").bold().dim(),
        ),
        &optional_progress_bar,
    );
    execute_script(
        project_root_path.join("cardano").as_path(),
        "deno",
        Vec::from(["run", "-A", "./aiken-to-lucid/src/main.ts"]),
        None,
    )?;
    log_or_show_progress(
        &format!(
            "{} 🚀 Starting Cardano services",
            style("Step 5/5").bold().dim(),
        ),
        &optional_progress_bar,
    );
    start_local_cardano_services(cardano_dir.as_path())?;

    log_or_show_progress(
        "🕦 Waiting for the Cardano services to start ...",
        &optional_progress_bar,
    );

    // TODO: make the url configurable
    let ogmios_url = "http://localhost:1337";
    let ogmios_connected =
        wait_for_health_check(ogmios_url, 20, 5000, None::<fn(&String) -> bool>).await;

    if ogmios_connected.is_ok() {
        verbose("✅ Cardano services started successfully");
    } else {
        return Err("❌ Failed to start Cardano services".into());
    }

    if config::get_config().cardano.services.db_sync {
        prepare_db_sync(cardano_dir.as_path())?;
    }
    seed_cardano_devnet(cardano_dir.as_path(), &optional_progress_bar);
    log_or_show_progress(
        "📄 Deploying the client, channel and connection contracts",
        &optional_progress_bar,
    );
    let handler_json_exists = wait_until_file_exists(
        project_root_path
            .join("cardano/deployments/handler.json")
            .as_path(),
        20,
        2000,
        || {
            let _ = execute_script(
                project_root_path.join("cardano").as_path(),
                "deno",
                Vec::from([
                    "run",
                    "--allow-net",
                    "--allow-env",
                    "--allow-read",
                    "--allow-write",
                    "src/deploy.ts",
                ]),
                None,
            );
        },
    );

    if handler_json_exists.is_ok() {
        if let Some(progress_bar) = &optional_progress_bar {
            progress_bar.finish_and_clear();
        }

        verbose("✅ Successully deployed the contracts");
        let options = fs_extra::file::CopyOptions::new().overwrite(true);
        std::fs::create_dir_all(project_root_path.join("cardano/gateway/src/deployment/"))?;
        copy(
            project_root_path.join("cardano/deployments/handler.json"),
            project_root_path.join("cardano/gateway/src/deployment/handler.json"),
            &options,
        )?;
        copy(
            project_root_path.join("cardano/deployments/handler.json"),
            project_root_path.join("relayer/examples/demo/configs/chains/chain_handler.json"),
            &options,
        )?;

        Ok(())
    } else {
        if let Some(progress_bar) = &optional_progress_bar {
            progress_bar.finish_and_clear();
        }
        Err("❌ Failed to start Cardano services. The handler.json file should have been created, but it doesn't exist. Consider running the start command again using --verbose 5.".into())
    }
}

pub async fn start_cosmos_sidechain(cosmos_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    execute_script(cosmos_dir, "docker", Vec::from(["compose", "stop"]), None)?;
    execute_script(
        cosmos_dir,
        "docker",
        Vec::from(["compose", "up", "-d", "--build"]),
        None,
    )?;

    let optional_progress_bar = match logger::get_verbosity() {
        logger::Verbosity::Verbose => None,
        _ => Some(ProgressBar::new_spinner()),
    };

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::with_template("{prefix:.bold} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        progress_bar.set_prefix(
            "🕦 Waiting for the Cosmos sidechain to start (this may take a while) ...".to_owned(),
        );
    } else {
        log("🕦 Waiting for the Cosmos sidechain to start ...");
    }

    // TODO: make the url configurable
    wait_for_health_check(
        "http://127.0.0.1:4500/",
        60,
        5000,
        None::<fn(&String) -> bool>,
    )
    .await?;

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.finish_and_clear();
    }

    Ok(())
}

pub fn start_local_cardano_services(cardano_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let configuration = config::get_config().cardano;

    let mut services = vec![];
    if configuration.services.cardano_node {
        services.push("cardano-node");
    }
    if configuration.services.postgres {
        services.push("postgres");
    }
    if configuration.services.kupo {
        services.push("kupo");
    }
    if configuration.services.ogmios {
        services.push("cardano-node-ogmios");
    }
    if configuration.services.db_sync {
        services.push("cardano-db-sync");
    }

    let mut script_stop_args = vec!["compose", "stop"];
    script_stop_args.append(&mut services.clone());
    execute_script(cardano_dir, "docker", script_stop_args, None)?;

    let mut script_start_args = vec!["compose", "up", "-d"];
    script_start_args.append(&mut services);
    execute_script(cardano_dir, "docker", script_start_args, None)?;
    Ok(())
}

pub async fn start_osmosis(osmosis_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let optional_progress_bar = match logger::get_verbosity() {
        logger::Verbosity::Verbose => None,
        _ => Some(ProgressBar::new_spinner()),
    };

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::with_template("{prefix:.bold} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        progress_bar.set_prefix("🥁‍ Starting Osmosis appchain ...".to_owned());
    } else {
        log("🥁‍ Starting Osmosis appchain ...");
    }

    let status = execute_script(
        osmosis_dir,
        "docker",
        Vec::from([
            "compose",
            "-f",
            "tests/localosmosis/docker-compose.yml",
            "up",
            "-d",
        ]),
        Some(Vec::from([(
            "OSMOSISD_CONTAINER_NAME",
            "localosmosis-osmosisd-1",
        )])),
    );

    if status.is_ok() {
        log_or_show_progress(
            "🚑 Waiting for the Osmosis appchain to become healthy ...",
            &optional_progress_bar,
        );

        // TODD: make the url and port configurable
        let is_healthy = wait_for_health_check(
            "http://127.0.0.1:26658/status?",
            30,
            3000,
            Some(|response_body: &String| {
                let json: Value = serde_json::from_str(&response_body).unwrap_or_default();

                if let Some(height) = json["result"]["sync_info"]["latest_block_height"]
                    .as_str()
                    .and_then(|h| h.parse::<u64>().ok())
                {
                    verbose(&format!("Current block height: {}", height));
                    return height > 0;
                }

                verbose(&format!(
                    "Failed to get the current block height from the response {}",
                    response_body,
                ));

                false
            }),
        )
        .await;

        if let Some(progress_bar) = &optional_progress_bar {
            progress_bar.finish_and_clear();
        }
        if is_healthy.is_ok() {
            Ok(())
        } else {
            Err("Run into timeout while checking http://127.0.0.1:26658/status?".into())
        }
    } else {
        if let Some(progress_bar) = &optional_progress_bar {
            progress_bar.finish_and_clear();
        }

        Err(status.unwrap_err().into())
    }
}

pub async fn prepare_osmosis(osmosis_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    check_osmosisd(osmosis_dir).await;
    match copy_osmosis_config_files(osmosis_dir) {
        Ok(_) => {
            verbose("✅ Osmosis configuration files copied successfully");
            remove_previous_chain_data()?;
            init_local_network(osmosis_dir)?;
            Ok(())
        }
        Err(e) => {
            error(&format!(
                "❌ Failed to copy Osmosis configuration files: {}",
                e
            ));
            Err(e.into())
        }
    }
}

pub fn configure_hermes(osmosis_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let optional_progress_bar = match logger::get_verbosity() {
        logger::Verbosity::Verbose => None,
        _ => Some(ProgressBar::new_spinner()),
    };

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::with_template("{prefix:.bold} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        progress_bar.set_prefix("🏃‍ Asking Hermes to connect Osmosis and Cosmos ...".to_owned());
    } else {
        log("🏃‍ Asking Hermes to connect Osmosis and Cosmos ...");
    }

    log_or_show_progress(
        &format!(
            "{} Prepare hermes configuration files and keys",
            style("Step 1/4").bold().dim()
        ),
        &optional_progress_bar,
    );

    let script_dir = osmosis_dir.join("scripts");
    if let Some(home_path) = home_dir() {
        let hermes_dir = home_path.join(".hermes");
        let options = fs_extra::file::CopyOptions::new().overwrite(true);
        verbose(&format!(
            "Copying Hermes configuration files from {} to {}",
            script_dir.join("hermes/config.toml").display(),
            hermes_dir.join("config.toml").display()
        ));
        copy(
            script_dir.join("hermes/config.toml"),
            hermes_dir.join("config.toml"),
            &options,
        )
        .expect("Failed to copy Hermes configuration file");
    }

    execute_script(
        script_dir.as_path(),
        "hermes",
        Vec::from([
            "keys",
            "add",
            "--overwrite",
            "--chain",
            "sidechain",
            "--mnemonic-file",
            osmosis_dir.join("scripts/hermes/cosmos").to_str().unwrap(),
        ]),
        None,
    )?;

    execute_script(
        script_dir.as_path(),
        "hermes",
        Vec::from([
            "keys",
            "add",
            "--overwrite",
            "--chain",
            "localosmosis",
            "--mnemonic-file",
            osmosis_dir.join("scripts/hermes/osmosis").to_str().unwrap(),
        ]),
        None,
    )?;

    log_or_show_progress(
        &format!(
            "{} Setup clients on both chains",
            style("Step 2/4").bold().dim()
        ),
        &optional_progress_bar,
    );

    let mut local_osmosis_client_id = None;
    for _ in 0..10 {
        // Try to create osmosis client
        let hermes_create_client_output = Command::new("hermes")
            .current_dir(&script_dir)
            .args(&[
                "create",
                "client",
                "--host-chain",
                "localosmosis",
                "--reference-chain",
                "sidechain",
            ])
            .output()
            .expect("Failed to create osmosis client");

        verbose(&format!(
            "status: {}, stdout: {}, stderr: {}",
            hermes_create_client_output.status,
            String::from_utf8_lossy(&hermes_create_client_output.stdout),
            String::from_utf8_lossy(&hermes_create_client_output.stderr)
        ));

        local_osmosis_client_id = extract_tendermint_client_id(hermes_create_client_output);

        if local_osmosis_client_id.is_none() {
            verbose("Failed to create client. Retrying in 5 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(5));
        } else {
            break;
        }
    }

    if let Some(local_osmosis_client_id) = local_osmosis_client_id {
        verbose(&format!(
            "localosmosis_client_id: {}",
            local_osmosis_client_id
        ));

        // Create sidechain client
        let create_sidechain_client_output = Command::new("hermes")
            .current_dir(&script_dir)
            .args(&[
                "create",
                "client",
                "--host-chain",
                "sidechain",
                "--reference-chain",
                "localosmosis",
                "--trusting-period",
                "86000s",
            ])
            .output()
            .expect("Failed to query clients");

        let sidechain_client_id = extract_tendermint_client_id(create_sidechain_client_output);

        if let Some(sidechain_client_id) = sidechain_client_id {
            verbose(&format!("sidechain_client_id: {}", sidechain_client_id));

            log_or_show_progress(
                &format!(
                    "{} Create a connection between both clients",
                    style("Step 3/4").bold().dim()
                ),
                &optional_progress_bar,
            );
            // Create connection
            let create_connection_output = Command::new("hermes")
                .current_dir(&script_dir)
                .args(&[
                    "create",
                    "connection",
                    "--a-chain",
                    "sidechain",
                    "--a-client",
                    sidechain_client_id.as_str(),
                    "--b-client",
                    &local_osmosis_client_id,
                ])
                .output()
                .expect("Failed to create connection");

            verbose(&format!(
                "status: {}, stdout: {}, stderr: {}",
                &create_connection_output.status,
                String::from_utf8_lossy(&create_connection_output.stdout),
                String::from_utf8_lossy(&create_connection_output.stderr)
            ));

            let connection_id = extract_tendermint_connection_id(create_connection_output);

            if let Some(connection_id) = connection_id {
                verbose(&format!("connection_id: {}", connection_id));

                // Create channel
                log_or_show_progress(
                    &format!("{} Create a channel", style("Step 4/4").bold().dim()),
                    &optional_progress_bar,
                );
                let create_channel_output = Command::new("hermes")
                    .current_dir(&script_dir)
                    .args(&[
                        "create",
                        "channel",
                        "--a-chain",
                        "sidechain",
                        "--a-connection",
                        &connection_id,
                        "--a-port",
                        "transfer",
                        "--b-port",
                        "transfer",
                    ])
                    .output()
                    .expect("Failed to query channels");

                if create_channel_output.status.success() {
                    verbose(&format!(
                        "{}",
                        String::from_utf8_lossy(&create_channel_output.stdout)
                    ));
                } else {
                    warn("Failed to get channel_id");
                }
            } else {
                warn("Failed to get connection_id");
            }
        } else {
            warn("Failed to get sidechain client_id");
        }
    } else {
        warn("Failed to get localosmosis client_id");
    }

    if let Some(progress_bar) = &optional_progress_bar {
        progress_bar.finish_and_clear();
    }

    Ok(())
}

fn init_local_network(osmosis_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if logger::is_quite() {
        execute_script(osmosis_dir, "make", Vec::from(["localnet-init"]), None)?;
        Ok(())
    } else {
        execute_script_with_progress(
            osmosis_dir,
            "make",
            Vec::from(["localnet-init"]),
            "Initialize local Osmosis network",
        )?;
        Ok(())
    }
}

fn remove_previous_chain_data() -> Result<(), fs_extra::error::Error> {
    if let Some(home_path) = home_dir() {
        let osmosis_data_dir = home_path.join(".osmosisd-local");
        if osmosis_data_dir.exists() {
            remove_dir_all(osmosis_data_dir)?;
            Ok(())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn copy_osmosis_config_files(osmosis_dir: &Path) -> Result<(), fs_extra::error::Error> {
    verbose(&format!(
        "Copying cosmwasm files from {} to {}",
        osmosis_dir.join("../configuration/cosmwasm/wasm").display(),
        osmosis_dir.join("cosmwasm").display()
    ));
    copy_items(
        &vec![osmosis_dir.join("../configuration/cosmwasm/wasm")],
        osmosis_dir.join("cosmwasm"),
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;

    verbose(&format!(
        "Copying hermes files from {} to {}",
        osmosis_dir.join("../configuration/hermes").display(),
        osmosis_dir.join("scripts").display()
    ));
    copy_items(
        &vec![osmosis_dir.join("../configuration/hermes")],
        osmosis_dir.join("scripts"),
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;

    let options = fs_extra::file::CopyOptions::new().overwrite(true);

    verbose(&format!(
        "Copying start.sh from {} to {}",
        osmosis_dir.join("../scripts/start.sh").display(),
        osmosis_dir.join("scripts/start.sh").display()
    ));
    copy(
        osmosis_dir.join("../scripts/start.sh"),
        osmosis_dir.join("scripts/start.sh"),
        &options,
    )?;

    verbose(&format!(
        "Copying stop.sh from {} to {}",
        osmosis_dir.join("../scripts/stop.sh").display(),
        osmosis_dir.join("scripts/stop.sh").display()
    ));
    copy(
        osmosis_dir.join("../scripts/stop.sh"),
        osmosis_dir.join("scripts/stop.sh"),
        &options,
    )?;

    verbose(&format!(
        "Copying setup_crosschain_swaps.sh from {} to {}",
        osmosis_dir
            .join("../scripts/setup_crosschain_swaps.sh")
            .display(),
        osmosis_dir
            .join("scripts/setup_crosschain_swaps.sh")
            .display()
    ));
    copy(
        osmosis_dir.join("../scripts/setup_crosschain_swaps.sh"),
        osmosis_dir.join("scripts/setup_crosschain_swaps.sh"),
        &options,
    )?;

    verbose(&format!(
        "Copying setup_osmosis_local.sh from {} to {}",
        osmosis_dir
            .join("../scripts/setup_osmosis_local.sh")
            .display(),
        osmosis_dir
            .join("tests/localosmosis/scripts/setup.sh")
            .display()
    ));
    copy(
        osmosis_dir.join("../scripts/setup_osmosis_local.sh"),
        osmosis_dir.join("tests/localosmosis/scripts/setup.sh"),
        &options,
    )?;

    verbose(&format!(
        "Copying docker-compose.yml from {} to {}",
        osmosis_dir
            .join("../configuration/docker-compose.yml")
            .display(),
        osmosis_dir
            .join("tests/localosmosis/docker-compose.yml")
            .display()
    ));
    copy(
        osmosis_dir.join("../configuration/docker-compose.yml"),
        osmosis_dir.join("tests/localosmosis/docker-compose.yml"),
        &options,
    )?;

    verbose(&format!(
        "Copying Dockerfile from {} to {}",
        osmosis_dir.join("../configuration/Dockerfile").display(),
        osmosis_dir.join("Dockerfile").display()
    ));
    copy(
        osmosis_dir.join("../configuration/Dockerfile"),
        osmosis_dir.join("Dockerfile"),
        &options,
    )?;

    Ok(())
}
