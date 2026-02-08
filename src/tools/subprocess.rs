/// A simple subprocess tool that executes external commands.
use std::process::Command;

use crate::{constants::DEFAULT_ENVS, utils};

fn get_cwd(args: &[String]) -> String {
    // Do not get the first argument.
    for arg in args[1..].iter() {
        if arg.starts_with("--trx8-subprocess-cwd=") {
            return arg["--trx8-subprocess-cwd=".len()..].to_string();
        }
    }
    std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn get_extra_envs(args: &[String]) -> Vec<(String, String)> {
    let mut envs = DEFAULT_ENVS.clone();
    for arg in args {
        if arg.starts_with("--trx8-subprocess-env=") {
            let env_pair = &arg["--trx8-subprocess-env=".len()..];
            if let Some((key, value)) = env_pair.split_once('=') {
                envs.push((key.to_string(), value.to_string()));
            }
        }
    }
    envs
}

pub fn cmd(args: &[String]) {
    let cwd = get_cwd(args);
    let command = &args[0];
    let output = match Command::new("cmd")
        .args(["/C", command])
        .envs(get_extra_envs(args))
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            tracing::error!("[cmd] Failed to execute command: {}", e);
            return;
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("[shell]: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("[shell] Command failed: {}", stderr);
    }
}

pub fn run(args: &[String]) {
    let command = &args[0];
    let cwd = get_cwd(args);
    let command_args: Vec<&String> = args
        .iter()
        .filter(|x| !x.starts_with("--trx8-subprocess"))
        .collect();

    let output = match Command::new(command)
        .args(command_args)
        .envs(get_extra_envs(args))
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            tracing::error!("[run] Failed to execute command: {}", e);
            return;
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("[run]: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("[run] Command failed: {}", stderr);
    }
}

#[cfg(target_os = "windows")]
pub fn ti_run(args: &[String]) {
    let command = &args[0];
    let cwd = get_cwd(args);
    let command_args: Vec<&String> = args
        .iter()
        .filter(|x| !x.starts_with("--trx8-subprocess"))
        .collect();

    match utils::nt::launch_as_ti(command.to_string(), command_args.iter().map(|s| s.to_string()).collect(), Some(cwd)) {
        true => {
            tracing::info!("[ti_run]: Command executed successfully.");
            return;
        }
        false => {
            tracing::error!("[ti_run] Failed to execute command as TrustedInstaller.");
            return;
        }
    };
}

pub fn pwsh(args: &[String]) {
    let command = &args[0];
    let cwd = get_cwd(args);
    let output = match Command::new("powershell")
        .args(["-Command", command])
        .envs(get_extra_envs(args))
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            tracing::error!("[pwsh] Failed to execute command: {}", e);
            return;
        }
    };
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("[pwsh]: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("[pwsh] Command failed: {}", stderr);
    }
}
