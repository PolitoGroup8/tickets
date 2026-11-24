use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    #[serde(rename = "not_scanned")]
    Unscanned,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum OrderMessage {
    OrderPaid { data: PaidOrder },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidOrder {
    pub(crate) order_id: String,
    pub(crate) customer_id: String,
    pub(crate) customer_email: String,
    pub(crate) customer_name: Option<String>,
    pub(crate) event: EventSnapshot,
    pub(crate) lines: Vec<PaidOrderLine>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidOrderLine {
    pub(crate) order_line_id: String,
    pub(crate) seat_id: String,
    pub(crate) seat_label: String,
    pub(crate) sector_id: String,
    #[serde(rename = "sector")]
    // nel json riceviamo sector ma per leggibilità del codice mettiamo sector_name
    pub(crate) sector_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NewTicket {
    pub(crate) order_line_id: String,
    pub(crate) seat_id: String,
    pub(crate) seat_label: String,
    pub(crate) sector_id: String,
    pub(crate) sector_name: String,
    pub(crate) status: TicketStatus,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EventSnapshot {
    pub(crate) event_id: String,
    pub(crate) session_id: String,
    pub(crate) event_title: String,
    pub(crate) starts_at: DateTime<Utc>,
    pub(crate) venue_name: String,
    pub(crate) venue_city: String,
}