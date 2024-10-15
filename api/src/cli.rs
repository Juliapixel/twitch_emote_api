use std::sync::LazyLock;

use clap::Parser;

pub static ARGS: LazyLock<Args> = LazyLock::new(|| {
    let _ = dotenvy::dotenv();
    Args::parse()
});

#[derive(clap::Parser)]
pub struct Args {
    /// Twitch App client ID
    #[arg(long, env = "TWITCH_CLIENT_ID", hide_env_values(true))]
    pub client_id: String,
    /// Twitch App client secret
    #[arg(long, env = "TWITCH_CLIENT_SECRET", hide_env_values(true))]
    pub client_secret: String,
    /// port to listen on
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}
