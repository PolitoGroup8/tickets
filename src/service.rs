use crate::domain::{NewTicket, OrderMessage, PaidOrder, TicketStatus};

fn issue_tickets(order: PaidOrder) -> Vec<NewTicket> {
    order
        .lines
        .into_iter()
        .map(|line| NewTicket {
            order_line_id: line.order_line_id,
            seat_id: line.seat_id,
            seat_label: line.seat_label,
            sector_id: line.sector_id,
            sector_name: line.sector_name,
            status: TicketStatus::Unscanned,
        })
        .collect()
}

pub(crate) fn issue_tickets_from_message(body: &[u8]) -> Result<Vec<NewTicket>, serde_json::Error> {
    let OrderMessage::OrderPaid { data } = serde_json::from_slice(body)?;
    Ok(issue_tickets(data))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::domain::{EventSnapshot, PaidOrderLine};

    use super::*;

    #[test]
    fn creates_one_unscanned_ticket_per_order_line() {
        let order = PaidOrder {
            order_id: "ordine-1".into(),
            customer_id: "cliente-1".into(),
            customer_email: "prova@prova.it".into(), // TODO: ricordarci di validare da qualche parte
            customer_name: None,
            event: EventSnapshot {
                event_id: "event-1".into(),
                session_id: "session-1".into(),
                event_title: "titolo-1".into(),
                starts_at: Utc::now(),
                venue_name: "venue-1".into(),
                venue_city: "torino-1".into(),
            },
            lines: vec![
                PaidOrderLine {
                    order_line_id: "line-1".into(),
                    seat_id: "seat-14".into(),
                    seat_label: "E14".into(),
                    sector_id: "s1".into(),
                    sector_name: "platea".into(),
                },
                PaidOrderLine {
                    order_line_id: "line-2".into(),
                    seat_id: "seat-15".into(),
                    seat_label: "E15".into(),
                    sector_id: "s2".into(),
                    sector_name: "piccionaia".into(),
                },
            ],
        };

        let tickets = issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        for (order_line_id, seat_id, seat_label, sector_id, sector_name) in [
            ("line-1", "seat-14", "E14", "s1", "platea"),
            ("line-2", "seat-15", "E15", "s2", "piccionaia"),
        ] {
            assert!(tickets.contains(&NewTicket {
                order_line_id: order_line_id.into(),
                seat_id: seat_id.into(),
                seat_label: seat_label.into(),
                sector_id: sector_id.into(),
                sector_name: sector_name.into(),
                status: TicketStatus::Unscanned,
            }));
        }

        println!("{tickets:#?}");
    }

    #[test]
    fn reads_paid_order_lines_from_json() {
        let json = r#"{
            "type": "OrderPaid",
            "version": 1,
            "data":{
                "orderId":"ordine-1",
                "customerId":"cliente-1",
                "customerEmail": "customer@example.test",
                "event": {
                    "eventId": "evt_test",
                    "sessionId": "ses_test",
                    "eventTitle": "Concerto",
                    "startsAt": "2026-12-10T20:00:00Z",
                    "venueName": "Teatro",
                    "venueCity": "Torino"
                },
                "lines":[
                    { "orderLineId": "line-1", "seatId": "seat-14", "seatLabel": "E14", "sectorId": "s1", "sector": "platea" },
                    { "orderLineId": "line-2", "seatId": "seat-15", "seatLabel": "E15", "sectorId": "s2", "sector": "piccionaia" }
                ]
            }
        }"#;

        let event: OrderMessage = serde_json::from_str(json).expect("valid OrderPaid message");
        let OrderMessage::OrderPaid { data: order } = event;

        assert_eq!(order.lines.len(), 2);
        assert_eq!(order.order_id, "ordine-1");
        assert_eq!(order.customer_id, "cliente-1");
        assert_eq!(order.lines[0].order_line_id, "line-1");
        assert_eq!(order.lines[0].seat_label, "E14");

        let tickets = issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        println!("{tickets:#?}");
    }

    #[test]
    fn rejects_order_cancelled_message() {
        let json = r#"{
            "type": "OrderCancelled",
            "version":1,
            "data":{
                "orderId": "ordine-1",
                "customerId": "cliente-1",
                "lines":[
                    { 
                        "orderLineId": "linea-1",
                        "seatId": "seat-14",
                        "seatLabel": "E14",
                        "sectorId": "s1",
                        "sector": "platea"
                    }
                ]
            }
        }"#;

        let result = serde_json::from_str::<OrderMessage>(json);

        assert!(result.is_err(), "OrderCancelled should be rejected");
    }
}
