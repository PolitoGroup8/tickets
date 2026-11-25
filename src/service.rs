use crate::{
    domain::{NewTicket, OrderMessage, PaidOrder, TicketStatus},
    repository::{Order, OrderLine, StoreError, TicketRepository, TicketTransaction},
    store::PgTickets,
};

pub(crate) async fn issue_tickets(
    order: PaidOrder,
    repository: &PgTickets,
) -> Result<Vec<NewTicket>, StoreError> {
    let mut tx: Box<dyn TicketTransaction> = repository.begin().await?;

    // TODO: Move this conversion into the adapter.
    let order = Order {
        order_id: order.order_id.into(),
        customer_id: order.customer_id.into(),

        lines: order
            .lines
            .into_iter()
            .map(|line| OrderLine {
                seat_id: line.seat_id,
                seat_label: line.seat_label,
                sector_id: line.sector_id,
                sector_name: line.sector_name,
            })
            .collect(),

        event_id: order.event.event_id.into(),
        event_title: order.event.event_title.into(),
        session_id: order.event.session_id.into(),
        session_starts_at: order.event.starts_at.into(),
        venue_name: order.event.venue_name.into(),
        venue_city: order.event.venue_city.into(),
        customer_email: order.customer_email.into(),
        customer_name: order.customer_name.into(),
    };

    tx.save_order(&order).await?;

    for line in &order.lines {
        tx.save_ticket(line).await?;
    }

    tx.commit().await?;

    let tickets = repository
        .find_by_order_id(&order.order_id)
        .await?
        .ok_or(StoreError::Failed(
            "Unreachable. The transaction committed".to_string(),
        ))?;

    Ok(tickets
        .into_iter()
        .map(|line| NewTicket {
            ticket_id: line.ticket_id,
            seat_id: line.seat_id,
            seat_label: line.seat_label,
            sector_id: line.sector_id,
            sector_name: line.sector_name,
            status: TicketStatus::Unscanned,
        })
        .collect())
}

pub(crate) async fn issue_tickets_from_message(
    body: &[u8],
    repository: &PgTickets,
) -> Result<Vec<NewTicket>, Box<dyn std::error::Error + Send + Sync>> {
    let OrderMessage::OrderPaid { data } = serde_json::from_slice(body)?;

    let tickets = issue_tickets(data, repository).await?;

    Ok(tickets)
}

#[cfg(test)]
fn test_issue_tickets(order: PaidOrder) -> Vec<NewTicket> {
    order
        .lines
        .into_iter()
        .map(|line| NewTicket {
            ticket_id: uuid::Uuid::new_v4().to_string(),
            seat_id: line.seat_id,
            seat_label: line.seat_label,
            sector_id: line.sector_id,
            sector_name: line.sector_name,
            status: TicketStatus::Unscanned,
        })
        .collect()
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
                    seat_id: "seat-14".into(),
                    seat_label: "E14".into(),
                    sector_id: "s1".into(),
                    sector_name: "platea".into(),
                },
                PaidOrderLine {
                    seat_id: "seat-15".into(),
                    seat_label: "E15".into(),
                    sector_id: "s2".into(),
                    sector_name: "piccionaia".into(),
                },
            ],
        };

        let tickets = test_issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        for (seat_id, seat_label, sector_id, sector_name) in [
            ("seat-14", "E14", "s1", "platea"),
            ("seat-15", "E15", "s2", "piccionaia"),
        ] {
            let ticket = tickets
                .iter()
                .find(|ticket| ticket.seat_id == seat_id)
                .expect("ticket for the expected seat");

            assert_eq!(ticket.seat_label, seat_label);
            assert_eq!(ticket.sector_id, sector_id);
            assert_eq!(ticket.sector_name, sector_name);
            assert_eq!(ticket.status, TicketStatus::Unscanned);
        }

        println!("{tickets:#?}");
    }

    #[test]
    fn reads_paid_order_lines_from_json() {
        let json = r#"{
            "type": "ORDER_PAID",
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
                    { "seatId": "seat-14", "seatLabel": "E14", "sectorId": "s1", "sector": "platea" },
                    { "seatId": "seat-15", "seatLabel": "E15", "sectorId": "s2", "sector": "piccionaia" }
                ]
            }
        }"#;

        let event: OrderMessage = serde_json::from_str(json).expect("valid OrderPaid message");
        let OrderMessage::OrderPaid { data: order } = event;

        assert_eq!(order.lines.len(), 2);
        assert_eq!(order.order_id, "ordine-1");
        assert_eq!(order.customer_id, "cliente-1");
        assert_eq!(order.lines[0].seat_id, "seat-14");
        assert_eq!(order.lines[0].seat_label, "E14");

        let tickets = test_issue_tickets(order);

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
