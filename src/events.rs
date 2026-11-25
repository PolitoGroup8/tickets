use chrono::{DateTime, Utc};
use serde_json::{Value, json};

pub(crate) fn tickets_issued_event(
    event_id: &str,
    occurred_at: DateTime<Utc>,
    order_id: &str,
    customer_email: &str,
    ticket_ids: &[String],
) -> Value {
    let tickets: Vec<_> = ticket_ids
        .iter()
        .map(|ticket_id| {
            json!({
                "ticketId": ticket_id,
                "qrObjectKey": crate::storage::ticket_qr_key(ticket_id),
            })
        })
        .collect();

    json!({
        "eventId": event_id,
        "type": "TICKETS_ISSUED",
        "version": 1,
        "producer": "tickets",
        "occurredAt": occurred_at,
        "data": {
            "orderId": order_id,
            "customerEmail": customer_email,
            "tickets": tickets,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_tickets_issued_payload() {
        let ticket_ids = ["ticket-1".to_owned(), "ticket-2".to_owned()];
        let occurred_at = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

        let event = tickets_issued_event(
            "event-1",
            occurred_at,
            "order-1",
            "customer@example.test",
            &ticket_ids,
        );

        assert_eq!(
            event,
            json!({
                "eventId": "event-1",
                "type": "TICKETS_ISSUED",
                "version": 1,
                "producer": "tickets",
                "occurredAt": "1970-01-01T00:00:00Z",
                "data": {
                    "orderId": "order-1",
                    "customerEmail": "customer@example.test",
                    "tickets": [
                        {
                            "ticketId": "ticket-1",
                            "qrObjectKey": "tickets/ticket-1/qr.png"
                        },
                        {
                            "ticketId": "ticket-2",
                            "qrObjectKey": "tickets/ticket-2/qr.png"
                        }
                    ]
                }
            })
        );
    }
}