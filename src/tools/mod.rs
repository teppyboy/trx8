mod console;

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
        _ => {
            tracing::warn!("Unknown action: {}", action_name);
        }
    }
}
