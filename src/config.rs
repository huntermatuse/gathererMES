use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// the connection url for the postgres database
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Url, // validates URL format automatically

    /// database connection pool size
    #[arg(long, env = "DB_POOL_SIZE", default_value = "50")]
    pub pool_size: u32,

    /// server bind address
    #[arg(long, env = "BIND_ADDRESS", default_value = "0.0.0.0:19080")]
    pub bind_address: String,

    // grpc bind address
    // TODO: make grpc work
    // #[clap(long, env = "GRPC_BIND_ADDRESS", default_value = "0.0.0.0:50051")]
    // pub grpc_bind_address: String,
    /// log level
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,
}

impl Config {
    /// validate the configuration after parsing
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.pool_size == 0 {
            anyhow::bail!("Pool size must be greater than 0");
        }

        if self.pool_size > 100 {
            tracing::warn!(
                "Pool size {} is quite large. PostgreSQL default max is 100 connections; suggest {}",
                self.pool_size,
                "97 max"
            );
        }

        Ok(())
    }
}
