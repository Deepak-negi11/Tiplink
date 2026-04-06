// insert user , find by email , find by id

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;
use crate::db::schema::users;


#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub balance: BigDecimal,
    pub is_active: Option<bool>,
    pub public_key: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub public_key : &'a str
}

impl User {
   
    pub fn signup(
        conn: &mut PgConnection,   
        email:&str,
        password:&str,
        public_key:&str
    ) -> QueryResult<User> {

        let new_user = NewUser{
            email,
            password,
            public_key
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }

    /// "Signin" - Find a user by their email to verify their password
    pub fn signin(
        conn: &mut PgConnection,
        user_email: &str,
    ) -> QueryResult<Option<User>> {
        users::table
            .filter(users::email.eq(user_email))
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

    
}