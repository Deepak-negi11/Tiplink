use dotenvy::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct IndexerConfig {
    pub grpc_endpoint: String,
    pub grpc_token: Option<String>,
    pub solana_rpc_url: String,
    pub database_url: String,
    pub tracked_programs: Vec<String>,
}

pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
pub const SPL_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

impl IndexerConfig {
    pub fn from_env() -> Self {
        dotenv().ok();

        let grpc_endpoint = env::var("GRPC_ENDPOINT")
            .unwrap_or_default();

        let grpc_token = env::var("GRPC_TOKEN").ok();

        let solana_rpc_url = env::var("SOLANA_RPC_URL")
            .expect("FATAL: SOLANA_RPC_URL must be set");

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let tracked_programs = vec![
            SYSTEM_PROGRAM.to_string(),
            SPL_TOKEN_PROGRAM.to_string(),
        ];

        Self {
            grpc_endpoint,
            grpc_token,
            solana_rpc_url,
            database_url,
            tracked_programs,
        }
    }
}
