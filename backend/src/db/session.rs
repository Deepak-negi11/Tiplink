use chrono::{DateTime, Utc};
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use uuid::Uuid;

use crate::db::schema::sessions;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
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
    pub user_id: Uuid,
    pub refresh_token: &'a str,
    pub device_info: Option<&'a str>,
    pub ip_address: Option<IpNetwork>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    /// INSERT refresh token session. Called at every login.
    pub fn create_session(
        conn: &mut PgConnection,
        user_id: Uuid,
        token_hash: &str,
        device: Option<&str>,
        ip: Option<IpNetwork>,
        expires_at: DateTime<Utc>,
    ) -> QueryResult<Uuid> {
        let new_session = NewSession {
            user_id,
            refresh_token: token_hash,
            device_info: device,
            ip_address: ip,
            expires_at,
        };

        diesel::insert_into(sessions::table)
            .values(&new_session)
            .returning(sessions::id)
            .get_result(conn)
    }

    /// SELECT session by hashed refresh token. Used at POST /auth/refresh.
    pub fn find_session(
        conn: &mut PgConnection,
        token_hash: &str,
    ) -> QueryResult<Option<Session>> {
        sessions::table
            .filter(sessions::refresh_token.eq(token_hash))
            .select(Session::as_select())
            .first(conn)
            .optional()
    }

    /// UPDATE revoked_at = NOW(). Called on logout.
    pub fn revoke_session(
        conn: &mut PgConnection,
        session_id: Uuid,
    ) -> QueryResult<()> {
        diesel::update(sessions::table.find(session_id))
            .set(sessions::revoked_at.eq(Utc::now()))
            .execute(conn)?;
        Ok(())
    }

    /// Revoke all sessions for a user. Called if account is compromised.
    pub fn revoke_all_sessions(
        conn: &mut PgConnection,
        user_id: Uuid,
    ) -> QueryResult<()> {
        diesel::update(sessions::table.filter(sessions::user_id.eq(user_id)))
            .set(sessions::revoked_at.eq(Utc::now()))
            .execute(conn)?;
        Ok(())
    }

    /// DELETE WHERE expires_at < NOW(). Run as a background job every hour.
    pub fn cleanup_expired(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(sessions::table.filter(sessions::expires_at.lt(Utc::now())))
            .execute(conn)
    }
}
