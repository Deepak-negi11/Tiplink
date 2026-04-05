// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "sign_status"))]
    pub struct SignStatus;
}

diesel::table! {
    access_log (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 20]
        action -> Varchar,
        requested_by_ip -> Nullable<Inet>,
        success -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dkg_sessions (session_id) {
        session_id -> Uuid,
        user_id -> Uuid,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 44]
        public_key -> Nullable<Varchar>,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    key_shares (id) {
        id -> Uuid,
        user_id -> Uuid,
        share_index -> Int4,
        encrypted_share -> Bytea,
        #[max_length = 50]
        encryption_key_id -> Varchar,
        created_at -> Timestamptz,
        last_accessed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SignStatus;

    sign_session (session_id) {
        session_id -> Uuid,
        user_id -> Uuid,
        tx_hash_to_sign -> Bytea,
        nonce -> Nullable<Bytea>,
        commitment -> Nullable<Bytea>,
        partial_sig -> Nullable<Bytea>,
        status -> SignStatus,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(access_log, dkg_sessions, key_shares, sign_session,);
