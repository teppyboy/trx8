use directories::ProjectDirs;
use std::sync::LazyLock;

pub static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("me", "tretrauit", "trx8").unwrap());

pub const DEFAULT_CONST_ENVS: &[(&str, &str)] = &[
    ("TRX8_VERSION", env!("CARGO_PKG_VERSION")),
    ("TRX8_REPOSITORY", env!("CARGO_PKG_REPOSITORY")),
];

pub static DEFAULT_ENVS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    // I do love shadowing variable names.
    let envs = vec![
        (
            "TRX8_WORKING_DIR".to_string(),
            std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        ),
        (
            "TRX8_USER_CACHE_DIR".to_string(),
            PROJECT_DIR.cache_dir().to_str().unwrap().to_string(),
        ),
        (
            "TRX8_USER_CONFIG_DIR".to_string(),
            PROJECT_DIR.config_dir().to_str().unwrap().to_string(),
        ),
        (
            "TRX8_USER_DATA_DIR".to_string(),
            PROJECT_DIR.data_dir().to_str().unwrap().to_string(),
        ),
    ];
    let envs = envs
        .into_iter()
        .chain(
            DEFAULT_CONST_ENVS
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        )
        .collect::<Vec<(String, String)>>();
    envs
});
