use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::users;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub balance: BigDecimal,
    pub is_active: bool,
    pub public_key: String,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub public_key: &'a str,
}

impl User {
    /// Create a new user entry
    pub fn signup(
        conn: &mut PgConnection,
        email: &str,
        password: &str,
        public_key: &str,
    ) -> QueryResult<User> {
        let new_user = NewUser {
            email,
            password,
            public_key,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }

    /// Find a user by their email
    pub fn find_by_email(
        conn: &mut PgConnection,
        email_val: &str,
    ) -> QueryResult<Option<User>> {
        users::table
            .filter(users::email.eq(email_val))
            .select(User::as_select())
            .first(conn)
            .optional()
    }

    /// Find a user by their ID
    pub fn find_by_id(
        conn: &mut PgConnection,
        user_id: Uuid,
    ) -> QueryResult<Option<User>> {
        users::table
            .find(user_id)
            .select(User::as_select())
            .first(conn)
            .optional()
    }

    /// Check if a user with the given email exists
    pub fn exists_by_email(
        conn: &mut PgConnection,
        email_val: &str,
    ) -> QueryResult<bool> {
        use diesel::dsl::{exists, select};
        select(exists(users::table.filter(users::email.eq(email_val))))
            .get_result(conn)
    }
}