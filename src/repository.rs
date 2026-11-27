use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{NewTicket, PaidOrder, PaidOrderLine, TicketStatus};

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    Failed(String),
    QrUploadFailed(String),
    OrderAlreadyExists,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Failed(msg) => write!(f, "{msg}"),
            StoreError::QrUploadFailed(msg) => write!(f, "QR upload failed: {msg}"),
            StoreError::OrderAlreadyExists => write!(f, "Order already exists"),
        }
    }
}

impl std::error::Error for StoreError {}

pub struct Order {
    pub order_id: String,
    pub customer_id: String,
    pub lines: Vec<OrderLine>,

    pub event_id: String,
    pub event_title: String,
    pub session_id: String,
    pub session_starts_at: DateTime<Utc>,
    pub venue_name: String,
    pub venue_city: String,
    pub customer_email: String,
    pub customer_name: Option<String>,
}

impl From<PaidOrder> for Order {
    fn from(order: PaidOrder) -> Self {
        Self {
            order_id: order.order_id.into(),
            customer_id: order.customer_id.into(),
            lines: order.lines.into_iter().map(Into::into).collect(),
            event_id: order.event.event_id.into(),
            event_title: order.event.event_title,
            session_id: order.event.session_id.into(),
            session_starts_at: order.event.starts_at,
            venue_name: order.event.venue_name,
            venue_city: order.event.venue_city,
            customer_email: order.customer_email,
            customer_name: order.customer_name,
        }
    }
}

pub struct OrderLine {
    pub seat_id: String,
    pub seat_label: String,
    pub sector_id: String,
    pub sector_name: String,
}

impl From<PaidOrderLine> for OrderLine {
    fn from(line: PaidOrderLine) -> Self {
        Self {
            seat_id: line.seat_id,
            seat_label: line.seat_label,
            sector_id: line.sector_id,
            sector_name: line.sector_name,
        }
    }
}

pub struct TicketRecord {
    pub ticket_id: String,
    pub meta_id: String,
    pub sector_id: String,
    pub sector_name: String,
    pub seat_id: String,
    pub seat_label: String,
    pub seat_row: Option<String>,
    pub seat_number: Option<i32>,
    pub issued_at: String,
    pub status: String,
}

impl TryFrom<TicketRecord> for NewTicket {
    type Error = StoreError;

    fn try_from(ticket: TicketRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            ticket_id: ticket.ticket_id,
            seat_id: ticket.seat_id,
            seat_label: ticket.seat_label,
            sector_id: ticket.sector_id,
            sector_name: ticket.sector_name,
            status: ticket.status.parse()?,
        })
    }
}

// Important design question: Do you need reads inside the transaction? Currently NO
// Important thing to keep in mind: Lifetime management with #[async_trait]

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TicketTransaction>, StoreError>;
    async fn find_by_order_id(
        &self,
        order_id: &str,
    ) -> Result<Option<Vec<TicketRecord>>, StoreError>;
    async fn find_by_ticket_id(&self, ticket_id: &str) -> Result<Option<TicketRecord>, StoreError>;
    async fn find_pending_tickets_issued(
    &self,
    order_id: &str,
) -> Result<Option<(uuid::Uuid, serde_json::Value)>, StoreError>;

async fn mark_tickets_issued_published(&self, event_id: uuid::Uuid) -> Result<(), StoreError>;
}

#[async_trait]
pub trait TicketTransaction: Send {
    async fn save_order(&mut self, order: &Order) -> Result<(), StoreError>;
    async fn save_ticket(
        &mut self,
        order_line: &OrderLine,
        credential_hash: &[u8; 32],
    ) -> Result<String, StoreError>;

    async fn save_tickets_issued(
        &mut self,
        event_id: uuid::Uuid,
        payload: &serde_json::Value,
    ) -> Result<(), StoreError>;

    async fn commit(self: Box<Self>) -> Result<(), StoreError>;
}
