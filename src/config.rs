use url::Url;

#[derive(clap::Parser)]
pub struct Config {
    /// the connection url for the postgres database
    #[clap(long, env = "DATABASE_URL")]
    pub database_url: Url, // validates URL format automatically
    
    /// database connection pool size
    #[clap(long, env = "DB_POOL_SIZE", default_value = "50")]
    pub pool_size: u32,
}