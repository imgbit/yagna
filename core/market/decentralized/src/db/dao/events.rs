use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thiserror::Error;

use ya_persistence::executor::Error as DbError;
use ya_persistence::executor::{do_with_transaction, AsDao, PoolType};

use crate::db::dao::demand::{demand_status, DemandState};
use crate::db::models::MarketEvent;
use crate::db::models::{OwnerType, Proposal, SubscriptionId};
use crate::db::schema::market_event::dsl;
use crate::db::DbResult;

#[derive(Error, Debug)]
pub enum TakeEventsError {
    #[error("Subscription [{0}] not found. Could be unsubscribed.")]
    SubscriptionNotFound(SubscriptionId),
    #[error("Subscription [{0}] expired.")]
    SubscriptionExpired(SubscriptionId),
    #[error("Failed to get events from database. Error: {0}.")]
    DatabaseError(DbError),
}

pub struct EventsDao<'c> {
    pool: &'c PoolType,
}

impl<'c> AsDao<'c> for EventsDao<'c> {
    fn as_dao(pool: &'c PoolType) -> Self {
        Self { pool }
    }
}

impl<'c> EventsDao<'c> {
    pub async fn add_proposal_event(&self, proposal: Proposal, owner: OwnerType) -> DbResult<()> {
        do_with_transaction(self.pool, move |conn| {
            let event = MarketEvent::from_proposal(&proposal, owner);
            diesel::insert_into(dsl::market_event)
                .values(event)
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    pub async fn take_requestor_events(
        &self,
        subscription_id: &SubscriptionId,
        max_events: i32,
    ) -> Result<Vec<MarketEvent>, TakeEventsError> {
        let subscription_id = subscription_id.clone();
        do_with_transaction(self.pool, move |conn| {
            match demand_status(conn, &subscription_id)? {
                DemandState::NotFound => Err(TakeEventsError::SubscriptionNotFound(
                    subscription_id.clone(),
                ))?,
                DemandState::Expired(_) => Err(TakeEventsError::SubscriptionExpired(
                    subscription_id.clone(),
                ))?,
                _ => (),
            };

            let events = dsl::market_event
                .filter(dsl::subscription_id.eq(&subscription_id))
                .order_by(dsl::timestamp.asc())
                .limit(max_events as i64)
                .load::<MarketEvent>(conn)?;

            // Remove returned events from queue.
            if !events.is_empty() {
                let ids = events.iter().map(|event| event.id).collect::<Vec<_>>();
                diesel::delete(dsl::market_event.filter(dsl::id.eq_any(ids))).execute(conn)?;
            }

            Ok(events)
        })
        .await
    }

    pub async fn remove_requestor_events(&self, subscription_id: &SubscriptionId) -> DbResult<()> {
        let subscription_id = subscription_id.clone();
        do_with_transaction(self.pool, move |conn| {
            diesel::delete(dsl::market_event.filter(dsl::subscription_id.eq(&subscription_id)))
                .execute(conn)?;
            Ok(())
        })
        .await
    }
}

impl<ErrorType: Into<DbError>> From<ErrorType> for TakeEventsError {
    fn from(err: ErrorType) -> Self {
        TakeEventsError::DatabaseError(err.into())
    }
}