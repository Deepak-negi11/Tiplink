use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ipnetwork::IpNetwork; 

use crate::db::schema::sessions;

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub device_info: Option<String>,
    pub ip_address: Option<IpNetwork>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: &'a str,
    pub device_info: Option<&'a str>,
    pub ip_address: Option<IpNetwork>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    /// 1. Create a new session when a user logs in
    pub fn create_session(
        conn: &mut PgConnection,
        new_session: NewSession,
    ) -> QueryResult<Session> {
        diesel::insert_into(sessions::table)
            .values(&new_session)
            .get_result(conn)
    }

    /// 2. SECURE LOOKUP: Find a session by token, BUT only if it is valid
    pub fn find_valid_by_token(
        conn: &mut PgConnection,
        token_val: &str,
    ) -> QueryResult<Option<Session>> {
        sessions::table
            .filter(sessions::refresh_token.eq(token_val))
            .filter(sessions::revoked_at.is_null())       
            .filter(sessions::expires_at.gt(Utc::now()))  
            .first(conn)
            .optional()
    }

    /// 3. Log a user out of a specific device
    pub fn revoke_session(
        conn: &mut PgConnection,
        session_id: Uuid,
    ) -> QueryResult<usize> {
        diesel::update(sessions::table.find(session_id))
            .set(sessions::revoked_at.eq(Some(Utc::now())))
            .execute(conn)
    }

    /// 4. SECURITY PANIC: Log a user out of ALL devices 
    pub fn revoke_all_for_user(
        conn: &mut PgConnection,
        target_user_id: Uuid,
    ) -> QueryResult<usize> {
        diesel::update(sessions::table)
            .filter(sessions::user_id.eq(target_user_id))
            .filter(sessions::revoked_at.is_null()) 
            .set(sessions::revoked_at.eq(Some(Utc::now())))
            .execute(conn)
    }
}