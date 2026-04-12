use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::payment_links;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::LinkStatus"]
#[diesel(postgres_type(name = "link_status"))]
pub enum LinkStatus {
    Active,
    Claimed,
    Expired,
    Cancelled,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = payment_links)]
pub struct PaymentLink {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub escrow_pda: String,
    pub claim_hash: String,
    pub token_mint: String,
    pub amount: i64,
    pub recipient_email: Option<String>,
    pub recipient_phone: Option<String>,
    pub status: LinkStatus,
    pub claimed_by: Option<Uuid>,
    pub claim_tx_hash: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = payment_links)]
pub struct NewPaymentLink<'a> {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub escrow_pda: &'a str,
    pub claim_hash: &'a str,
    pub token_mint: &'a str,
    pub amount: i64,
    pub recipient_email: Option<&'a str>,
    pub recipient_phone: Option<&'a str>,
    pub status: LinkStatus,
    pub expires_at: DateTime<Utc>,
    pub memo: Option<&'a str>,
}

impl PaymentLink {
    /// Create a new payment link entry
    pub fn create_link(
        conn: &mut PgConnection,
        new_link: NewPaymentLink,
    ) -> QueryResult<PaymentLink> {
        diesel::insert_into(payment_links::table)
            .values(&new_link)
            .get_result(conn)
    }

    /// Find a link by its UUID
    pub fn find_by_id(
        conn: &mut PgConnection,
        link_id: Uuid,
    ) -> QueryResult<Option<PaymentLink>> {
        payment_links::table
            .find(link_id)
            .select(PaymentLink::as_select())
            .first(conn)
            .optional()
    }

    /// Find a link by its claim hash (used when claiming)
    pub fn find_by_hash(
        conn: &mut PgConnection,
        hash: &str,
    ) -> QueryResult<Option<PaymentLink>> {
        payment_links::table
            .filter(payment_links::claim_hash.eq(hash))
            .select(PaymentLink::as_select())
            .first(conn)
            .optional()
    }

    /// Mark a link as claimed
    pub fn mark_as_claimed(
        conn: &mut PgConnection,
        link_id: Uuid,
        claimer_id: Uuid,
        tx_hash: &str,
    ) -> QueryResult<usize> {
        diesel::update(payment_links::table.find(link_id))
            .set((
                payment_links::status.eq(LinkStatus::Claimed),
                payment_links::claimed_by.eq(claimer_id),
                payment_links::claim_tx_hash.eq(tx_hash),
                payment_links::claimed_at.eq(Utc::now()),
            ))
            .execute(conn)
    }

    /// Mark a link as expired if current time is past expires_at
    pub fn expire_links(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(
            payment_links::table
                .filter(payment_links::status.eq(LinkStatus::Active))
                .filter(payment_links::expires_at.lt(Utc::now())),
        )
        .set(payment_links::status.eq(LinkStatus::Expired))
        .execute(conn)
    }

    /// Get all links created by a specific user
    pub fn get_user_links(
        conn: &mut PgConnection,
        user_id: Uuid,
    ) -> QueryResult<Vec<PaymentLink>> {
        payment_links::table
            .filter(payment_links::creator_id.eq(user_id))
            .select(PaymentLink::as_select())
            .load(conn)
    }
}