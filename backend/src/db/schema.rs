
pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "link_status"))]
    pub struct LinkStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "swap_status"))]
    pub struct SwapStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tx_type"))]
    pub struct TxType;
}

diesel::table! {
    balances (id) {
        id -> Uuid,
        amount -> Int8,
        user_id -> Uuid,
        #[max_length = 44]
        token_mint -> Varchar,
        #[max_length = 44]
        token_symbol -> Varchar,
        locked -> Int8,
        available -> Int8,
        decimals -> Int2,
        updated_at -> Timestamptz,
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
        user_id -> Uuid,
        refresh_token -> Text,
        device_info -> Nullable<Text>,
        ip_address -> Nullable<Inet>,
        revoked_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SwapStatus;

    swap_history (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 44]
        input_mint -> Varchar,
        #[max_length = 44]
        output_mint -> Varchar,
        output_amount -> Int8,
        input_amount -> Int8,
        fee_amount -> Int8,
        price_impact -> Numeric,
        #[max_length = 88]
        tx_hash -> Varchar,
        status -> SwapStatus,
        created_at -> Timestamptz,
        confirmed_at -> Nullable<Timestamptz>,
        requested_slippage_bps -> Int4,
    }
}

diesel::table! {
    transaction_intents (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        intent_message -> Text,
        intent_signature -> Text,
        unsigned_payload -> Nullable<Text>,
        #[max_length = 50]
        status -> Nullable<Varchar>,
        final_tx_hash -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TxType;

    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        amount -> Int8,
        #[max_length = 44]
        token_mint -> Varchar,
        #[max_length = 44]
        token_symbol -> Varchar,
        #[max_length = 88]
        tx_hash -> Varchar,
        tx_type -> TxType,
        #[max_length = 44]
        from_address -> Varchar,
        #[max_length = 44]
        to_address -> Varchar,
        slot -> Int8,
        block_time -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        password -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        balance -> Numeric,
        is_active -> Bool,
        #[max_length = 44]
        public_key -> Varchar,
    }
}

diesel::joinable!(balances -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(swap_history -> users (user_id));
diesel::joinable!(transaction_intents -> users (user_id));
diesel::joinable!(transactions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    payment_links,
    sessions,
    swap_history,
    transaction_intents,
    transactions,
    users,
);
