
use diesel::prelude::*;
use diesel::PgConnection;
use diesel::result::ConnectionError;
use std::env;

pub struct Store {
    pub conn: PgConnection 
}


impl Store {
   
    pub fn new() -> Result<Self, ConnectionError> {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| panic!("DATABASE_URL environment variable must be set"));
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }
}