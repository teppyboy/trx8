use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub metadata: Metadata,
    pub scripts: Vec<Script>,
}

impl Config {
    pub fn generate_empty() -> Self {
        Config {
            metadata: Metadata {
                name: "Trx8".to_string(),
                author: None,
                version: "0.1.0".to_string(),
                trx8_version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some("This is an example configuration profile.".to_string()),
            },
            scripts: vec![Script {
                name: "An example script".to_string(),
                condition: None,
                description: Some("This script does nothing :)".to_string()),
                actions: vec![Action {
                    name: "echo".to_string(),
                    description: Some(
                        "Prints Hello, World! to the console because why not?".to_string(),
                    ),
                    parameters: Some(vec!["Hello, World!".to_string()]),
                }],
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub trx8_version: String,
    pub description: Option<String>,
    pub author: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub name: String,
    pub condition: Option<String>,
    pub description: Option<String>,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Vec<String>>,
}
