use async_trait::async_trait;

use crate::{domain::{PaidOrder, PaidOrderLine}};

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    Failed(String),
}

// Important design question: Do you need reads inside the transaction? Currently NO
// Important thing to keep in mind: Lifetime management with #[async_trait]

/*
#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn begin(&self) -> Result<Box<todo!()>, StoreError>;
    async fn find_by_order_id(&self, order_id: &str) -> Result<Option<todo!()>, StoreError>;
    async fn find_by_ticket_id(&self, ticket_id: &str) -> Result<Option<todo!()>, StoreError>;
}

pub trait TicketTransaction: Send {
    async fn save_order(&mut self, order: &PaidOrder) -> Result<todo!(), StoreError>;
    async fn save_ticket(&mut self, order_line: &PaidOrderLine) -> Result<todo!(), StoreError>;
    async fn commit(self: Box<Self>) -> Result<(), StoreError>;
}
*/
