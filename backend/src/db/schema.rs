// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "link_status"))]
    pub struct LinkStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "swap_staus"))]
    pub struct SwapStaus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tx_type"))]
    pub struct TxType;
}

diesel::table! {
    balances (id) {
        id -> Uuid,
        amount -> Nullable<Int8>,
        user_id -> Nullable<Uuid>,
        #[max_length = 44]
        token_mint -> Nullable<Varchar>,
        #[max_length = 44]
        token_symbol -> Nullable<Varchar>,
        locked -> Nullable<Int8>,
        available -> Nullable<Int8>,
        decimals -> Nullable<Int2>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LinkStatus;

    payment_links (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 44]
        escrow_pda -> Varchar,
        #[max_length = 64]
        claim_hash -> Varchar,
        #[max_length = 44]
        token_mint -> Varchar,
        amount -> Int8,
        #[max_length = 255]
        recipient_email -> Nullable<Varchar>,
        #[max_length = 20]
        recipient_phone -> Nullable<Varchar>,
        status -> LinkStatus,
        claimed_by -> Nullable<Uuid>,
        #[max_length = 88]
        claim_tx_hash -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        memo -> Nullable<Text>,
        created_at -> Timestamptz,
        claimed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        refresh_token -> Text,
        device_info -> Nullable<Text>,
        ip_address -> Nullable<Inet>,
        revoked_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SwapStaus;

    swap_history (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 44]
        input_mint -> Nullable<Varchar>,
        #[max_length = 44]
        output_mint -> Nullable<Varchar>,
        output_amount -> Nullable<Int8>,
        input_amount -> Nullable<Int8>,
        fee_amount -> Nullable<Int8>,
        price_impact -> Nullable<Numeric>,
        #[max_length = 88]
        tx_hash -> Nullable<Varchar>,
        status -> Nullable<SwapStaus>,
        created_at -> Nullable<Timestamptz>,
        confirmed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TxType;

    transactions (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        amount -> Nullable<Int8>,
        #[max_length = 44]
        token_mint -> Nullable<Varchar>,
        #[max_length = 44]
        token_symbol -> Nullable<Varchar>,
        #[max_length = 88]
        tx_hash -> Nullable<Varchar>,
        tx_type -> Nullable<TxType>,
        #[max_length = 44]
        from_address -> Nullable<Varchar>,
        #[max_length = 44]
        to_address -> Nullable<Varchar>,
        slot -> Nullable<Int8>,
        block_time -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        password -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        balance -> Numeric,
        is_active -> Nullable<Bool>,
        #[max_length = 44]
        public_key -> Varchar,
    }
}

diesel::joinable!(balances -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(swap_history -> users (user_id));
diesel::joinable!(transactions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    payment_links,
    sessions,
    swap_history,
    transactions,
    users,
);
