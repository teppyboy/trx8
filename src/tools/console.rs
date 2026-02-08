/// A simple console tool that prints messages to the console.
use tracing::info;

pub fn echo(message: &str) {
    info!("[echo]: {}", message);
}
