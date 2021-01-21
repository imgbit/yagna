/*
    Database Access Object, all you need to interact with the database.
*/

// Extrernal crates
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Workspace uses
use ya_payment_driver::{
    dao::{payment::PaymentDao, transaction::TransactionDao, DbExecutor},
    db::models::{
        PaymentEntity, TransactionEntity, TransactionStatus, PAYMENT_STATUS_FAILED,
        PAYMENT_STATUS_NOT_YET, TX_CREATED,
    },
    model::{GenericError, PaymentDetails, SchedulePayment},
    utils,
};

pub struct ZksyncDao {
    db: DbExecutor,
}

impl ZksyncDao {
    pub fn new(db: DbExecutor) -> Self {
        Self { db }
    }

    fn payment(&self) -> PaymentDao {
        self.db.as_dao::<PaymentDao>()
    }

    fn transaction(&self) -> TransactionDao {
        self.db.as_dao::<TransactionDao>()
    }

    pub async fn get_pending_payments(&self, node_id: &str) -> Vec<PaymentEntity> {
        match self
            .payment()
            .get_pending_payments(node_id.to_string())
            .await
        {
            Ok(payments) => payments,
            Err(e) => {
                log::error!(
                    "Failed to fetch pending payments for {:?} : {:?}",
                    node_id,
                    e
                );
                vec![]
            }
        }
    }

    pub async fn insert_payment(&self, order_id: &str, msg: &SchedulePayment) {
        let recipient = msg.recipient().to_owned();
        let gnt_amount = utils::big_dec_to_u256(msg.amount());
        let gas_amount = Default::default();

        let payment = PaymentEntity {
            amount: utils::u256_to_big_endian_hex(gnt_amount),
            gas: utils::u256_to_big_endian_hex(gas_amount),
            order_id: order_id.to_string(),
            payment_due_date: msg.due_date().naive_utc(),
            sender: msg.sender().clone(),
            recipient: recipient.clone(),
            status: PAYMENT_STATUS_NOT_YET,
            tx_id: None,
        };
        if let Err(e) = self.payment().insert(payment).await {
            log::error!(
                "Failed to store transaction for {:?} , msg={:?}, err={:?}",
                order_id,
                msg,
                e
            )
            // TO CHECK: Should it continue or stop the process...
        }
    }

    pub async fn insert_transaction(
        &self,
        details: &PaymentDetails,
        date: DateTime<Utc>,
    ) -> String {
        // TO CHECK: No difference between tx_id and tx_hash on zksync
        // TODO: Implement pre-sign
        let tx_id = Uuid::new_v4().to_string();
        let tx = TransactionEntity {
            tx_id: tx_id.clone(),
            sender: details.sender.clone(),
            nonce: "".to_string(), // not used till pre-sign
            status: TX_CREATED,
            timestamp: date.naive_utc(),
            tx_type: 0,                // Zksync only knows transfers, unused field
            encoded: "".to_string(),   // not used till pre-sign
            signature: "".to_string(), // not used till pre-sign
            tx_hash: None,
        };

        if let Err(e) = self.transaction().insert_transactions(vec![tx]).await {
            log::error!("Failed to store transaction for {:?} : {:?}", details, e)
            // TO CHECK: Should it continue or stop the process...
        }
        tx_id
    }

    pub async fn transaction_confirmed(&self, tx_id: &str, result: bool) -> Vec<PaymentEntity> {
        let status = if result {
            TransactionStatus::Confirmed
        } else {
            TransactionStatus::Failed
        };

        if let Err(e) = self
            .transaction()
            .update_tx_status(tx_id.to_string(), status.into())
            .await
        {
            log::error!("Failed to update tx status for {:?} : {:?}", tx_id, e)
            // TO CHECK: Should it continue or stop the process...
        }
        if result {
            match self.payment().get_by_tx_id(tx_id.to_string()).await {
                Ok(payments) => return payments,
                Err(e) => log::error!("Failed to fetch `payments` for tx {:?} : {:?}", tx_id, e),
            };
        }
        vec![]
    }

    pub async fn transaction_success(&self, tx_id: &str, tx_hash: &str, order_id: &str) {
        if let Err(e) = self
            .payment()
            .update_tx_id(order_id.to_string(), tx_id.to_string())
            .await
        {
            log::error!("Failed to update for transaction {:?} : {:?}", tx_id, e)
            // TO CHECK: Should it continue or stop the process...
        }
        if let Err(e) = self
            .transaction()
            .update_tx_sent(tx_id.to_string(), tx_hash.to_string())
            .await
        {
            log::error!("Failed to update for transaction {:?} : {:?}", tx_id, e)
            // TO CHECK: Should it continue or stop the process...
        }
    }

    pub async fn transaction_failed(&self, tx_id: &str, error: &GenericError, order_id: &str) {
        if let Err(e) = self
            .payment()
            .update_status(
                order_id.to_string(),
                match error {
                    // TODO: Handle other statusses
                    // GNTDriverError::InsufficientFunds => PAYMENT_STATUS_NOT_ENOUGH_FUNDS,
                    // GNTDriverError::InsufficientGas => PAYMENT_STATUS_NOT_ENOUGH_GAS,
                    _ => PAYMENT_STATUS_FAILED,
                },
            )
            .await
        {
            log::error!(
                "Failed to update transaction failed in `payment` {:?} : {:?}",
                tx_id,
                e
            )
            // TO CHECK: Should it continue or stop the process...
        }

        if let Err(e) = self
            .transaction()
            .update_tx_status(tx_id.to_string(), TransactionStatus::Failed.into())
            .await
        {
            log::error!(
                "Failed to update transaction failed in `transaction` {:?} : {:?}",
                tx_id,
                e
            )
            // TO CHECK: Should it continue or stop the process...
        }
    }

    pub async fn get_unconfirmed_txs(&self) -> Vec<TransactionEntity> {
        match self.transaction().get_unconfirmed_txs().await {
            Ok(txs) => txs,
            Err(e) => {
                log::error!("Failed to fetch unconfirmed transactions : {:?}", e);
                vec![]
            }
        }
    }
}