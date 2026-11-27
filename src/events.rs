use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::domain::{EventSnapshot, NewTicket};

pub(crate) fn tickets_issued_event<'a>(
    event_id: &str,
    occurred_at: DateTime<Utc>,
    order_id: &str,
    customer_email: &str,
    customer_name: Option<&str>,
    event_snapshot: &EventSnapshot,
    tickets: impl IntoIterator<Item = &'a NewTicket>,
) -> Value {
    let tickets: Vec<_> = tickets
        .into_iter()
        .map(|ticket| {
            json!({
                "ticketId": ticket.ticket_id.as_str(),
                "qrObjectKey": crate::storage::ticket_qr_key(&ticket.ticket_id),
                "seatLabel": ticket.seat_label.as_str(),
                "sector": ticket.sector_name.as_str(),
            })
        })
        .collect();

    let mut data = json!({
        "orderId": order_id,
        "customerEmail": customer_email,
        "event": {
            "eventTitle": event_snapshot.event_title,
            "startsAt": event_snapshot.starts_at,
            "venueName": event_snapshot.venue_name,
            "venueCity": event_snapshot.venue_city,
        },
        "tickets": tickets,
    });

    if let Some(customer_name) = customer_name {
        data["customerName"] = json!(customer_name);
    }

    json!({
        "eventId": event_id,
        "type": "TICKETS_ISSUED",
        "version": 1,
        "producer": "tickets",
        "occurredAt": occurred_at,
        "data": data,
    })
}
#[cfg(test)]
mod tests {
    use crate::domain::TicketStatus;

use super::*;
    use chrono::TimeZone;

    #[test]
    fn builds_tickets_issued_payload() {
        let occurred_at = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

        let starts_at = Utc
            .with_ymd_and_hms(2026, 12, 10, 20, 0, 0)
            .unwrap();

        let event_snapshot = EventSnapshot {
            event_id: "event-001".into(),
            session_id: "session-001".into(),
            event_title: "Concerto".into(),
            starts_at,
            venue_name: "Teatro".into(),
            venue_city: "Torino".into(),
        };

        let tickets = vec![
            NewTicket {
                ticket_id: "ticket-1".into(),
                seat_id: "seat-1".into(),
                seat_label: "E14".into(),
                sector_id: "sector-1".into(),
                sector_name: "platea".into(),
                status: TicketStatus::Unscanned,
            },
            NewTicket {
                ticket_id: "ticket-2".into(),
                seat_id: "seat-2".into(),
                seat_label: "E15".into(),
                sector_id: "sector-1".into(),
                sector_name: "platea".into(),
                status: TicketStatus::Unscanned,
            },
        ];

        let event = tickets_issued_event(
            "event-1",
            occurred_at,
            "order-1",
            "customer@example.test",
            Some("Mario Rossi"),
            &event_snapshot,
            &tickets,
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
                    "customerName": "Mario Rossi",
                    "event": {
                        "eventTitle": "Concerto",
                        "startsAt": "2026-12-10T20:00:00Z",
                        "venueName": "Teatro",
                        "venueCity": "Torino"
                    },
                    "tickets": [
                        {
                            "ticketId": "ticket-1",
                            "qrObjectKey": "tickets/ticket-1/qr.png",
                            "seatLabel": "E14",
                            "sector": "platea"
                        },
                        {
                            "ticketId": "ticket-2",
                            "qrObjectKey": "tickets/ticket-2/qr.png",
                            "seatLabel": "E15",
                            "sector": "platea"
                        }
                    ]
                }
            })
        );
    }

    #[test]
    fn omits_customer_name_when_missing() {
        let occurred_at = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

        let event_snapshot = EventSnapshot {
            event_id: "event-001".into(),
            session_id: "session-001".into(),
            event_title: "Concerto".into(),
            starts_at: Utc.with_ymd_and_hms(2026, 12, 10, 20, 0, 0).unwrap(),
            venue_name: "Teatro".into(),
            venue_city: "Torino".into(),
        };

        let event = tickets_issued_event(
            "event-1",
            occurred_at,
            "order-1",
            "customer@example.test",
            None,
            &event_snapshot,
            &[],
        );

        assert!(event["data"].get("customerName").is_none());
        assert_eq!(event["data"]["tickets"], json!([]));
    }
}