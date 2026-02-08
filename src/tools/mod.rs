mod console;
mod subprocess;

pub fn execute_action(action_name: &str, parameters: &Option<Vec<String>>) {
    match action_name {
        "echo" => {
            if let Some(params) = parameters {
                if !params.is_empty() {
                    console::echo(&params[0]);
                } else {
                    console::echo("No message provided, please check your configuration.");
                }
            } else {
                console::echo("No message provided, please check your configuration.");
            }
        }
        "cmd" => {
            if let Some(params) = parameters {
                if !params.is_empty() {
                    subprocess::cmd(params);
                } else {
                    tracing::warn!("[cmd] No command provided, please check your configuration.");
                }
            } else {
                tracing::warn!("[cmd] No command provided, please check your configuration.");
            }
        }
        "pwsh" => {
            if let Some(params) = parameters {
                if !params.is_empty() {
                    subprocess::pwsh(params);
                } else {
                    tracing::warn!("[pwsh] No command provided, please check your configuration.");
                }
            } else {
                tracing::warn!("[pwsh] No command provided, please check your configuration.");
            }
        }
        "run" => {
            if let Some(params) = parameters {
                if !params.is_empty() {
                    subprocess::run(params);
                } else {
                    tracing::warn!("[run] No command provided, please check your configuration.");
                }
            } else {
                tracing::warn!("[run] No command provided, please check your configuration.");
            }
        }
        #[cfg(target_os = "windows")]
        "ti_run" => {
            if let Some(params) = parameters {
                if !params.is_empty() {
                    subprocess::ti_run(params);
                } else {
                    tracing::warn!("[ti_run] No command provided, please check your configuration.");   
                }
            } else {
                tracing::warn!("[ti_run] No command provided, please check your configuration.");
            }
        }
        _ => {
            tracing::warn!("Unknown action: {}", action_name);
        }
    }
}
