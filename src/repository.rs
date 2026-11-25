use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    Failed(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Failed(msg) => write!(f, "{msg}"),
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

pub struct OrderLine {
    pub seat_id: String,
    pub seat_label: String,
    pub sector_id: String,
    pub sector_name: String,
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

// Important design question: Do you need reads inside the transaction? Currently NO
// Important thing to keep in mind: Lifetime management with #[async_trait]

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TicketTransaction>, StoreError>;
    async fn find_by_order_id(&self, order_id: &str) -> Result<Option<Vec<TicketRecord>>, StoreError>;
    async fn find_by_ticket_id(&self, ticket_id: &str) -> Result<Option<TicketRecord>, StoreError>;
}

#[async_trait]
pub trait TicketTransaction: Send {
    async fn save_order(&mut self, order: &Order) -> Result<(), StoreError>;
    async fn save_ticket(&mut self, order_line: &OrderLine) -> Result<(), StoreError>;
    async fn commit(self: Box<Self>) -> Result<(), StoreError>;
}

/// The current time.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The clock of the running service.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
