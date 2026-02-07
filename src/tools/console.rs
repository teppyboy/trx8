use tracing::info;

pub fn echo(message: &str) {
    info!("[echo]: {}", message);
}
