use directories::ProjectDirs;
use std::sync::LazyLock;

pub static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("me", "tretrauit", "trx8").unwrap());
