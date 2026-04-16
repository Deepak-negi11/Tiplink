use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::Insertable;
use bigdecimal::BigDecimal;
use crate::db::schema::users;

/// What Next.js sends to `POST /signup`
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String, 
    pub user_id: Uuid,
    pub email: String,
    pub public_key: String, 
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password: &'a str, 
    pub public_key: &'a str,
    pub balance: bigdecimal::BigDecimal, 
    pub is_active: bool,
}