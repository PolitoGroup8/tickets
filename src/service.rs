use crate::{
    domain::{NewTicket, OrderMessage, PaidOrder, TicketStatus},
    repository::{Order, OrderLine, StoreError, TicketRepository, TicketTransaction},
    store::PgTickets,
};
use sha2::{Digest, Sha256};
use std::future::Future;
use uuid::Uuid;

struct TicketCredential {
    token: Uuid,
    hash: [u8; 32],
}

fn generate_ticket_credential() -> TicketCredential {
    let token = Uuid::new_v4();
    let hash = Sha256::digest(token.as_bytes()).into();

    TicketCredential { token, hash }
}

struct IssuedTicket {
    ticket: NewTicket,
    token: Uuid,
}

pub(crate) async fn issue_tickets<Fut>(
    order: PaidOrder,
    repository: &PgTickets,
    upload_qr: impl Fn(String, Uuid) -> Fut,
) -> Result<Vec<NewTicket>, StoreError>
where
    Fut: Future<Output = Result<(), StoreError>>,
{
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

    match tx.save_order(&order).await {
        Ok(()) => {}
        Err(StoreError::OrderAlreadyExists) => {
            drop(tx);
            return load_existing_tickets(repository, &order).await;
        }
        Err(err) => return Err(err),
    }

    let mut tickets = Vec::with_capacity(order.lines.len());

    for line in order.lines {
        let credential = generate_ticket_credential();
        let ticket_id = tx.save_ticket(&line, &credential.hash).await?;

        tickets.push(IssuedTicket {
            ticket: NewTicket {
                ticket_id,
                seat_id: line.seat_id,
                seat_label: line.seat_label,
                sector_id: line.sector_id,
                sector_name: line.sector_name,
                status: TicketStatus::Unscanned,
            },
            token: credential.token,
        });
    }

    for issued in &tickets {
        upload_qr(issued.ticket.ticket_id.clone(), issued.token).await?;
    }

    let event_id = Uuid::new_v4();
    let ticket_ids: Vec<_> = tickets
        .iter()
        .map(|issued| issued.ticket.ticket_id.clone())
        .collect();

    let event = crate::events::tickets_issued_event(
        &event_id.to_string(),
        chrono::Utc::now(),
        &order.order_id,
        &order.customer_email,
        &ticket_ids,
    );
    tx.save_tickets_issued(event_id, &event).await?;
    tx.commit().await?;
    Ok(tickets.into_iter().map(|issued| issued.ticket).collect())
}

async fn load_existing_tickets(
    repository: &PgTickets,
    order: &Order,
) -> Result<Vec<NewTicket>, StoreError> {
    let stored = repository
        .find_by_order_id(&order.order_id)
        .await?
        .unwrap_or_default();

    let mut expected_seats: Vec<_> = order.lines.iter().map(|line| &line.seat_id).collect();
    let mut stored_seats: Vec<_> = stored.iter().map(|ticket| &ticket.seat_id).collect();
    expected_seats.sort_unstable();
    stored_seats.sort_unstable();

    if expected_seats != stored_seats {
        return Err(StoreError::Failed(
            "Order already exists with different seats".into(),
        ));
    }

    stored
        .into_iter()
        .map(|ticket| {
            let status = match ticket.status.as_str() {
                "UNSCANNED" => TicketStatus::Unscanned,
                "SCANNED" => TicketStatus::Scanned,
                "REVOKED" => TicketStatus::Revoked,
                other => {
                    return Err(StoreError::Failed(format!(
                        "Unknown ticket status: {other}"
                    )));
                }
            };
            Ok(NewTicket {
                ticket_id: ticket.ticket_id,
                seat_id: ticket.seat_id,
                seat_label: ticket.seat_label,
                sector_id: ticket.sector_id,
                sector_name: ticket.sector_name,
                status,
            })
        })
        .collect()
}

pub(crate) async fn issue_tickets_from_message<Fut>(
    body: &[u8],
    repository: &PgTickets,
    upload_qr: impl Fn(String, Uuid) -> Fut,
) -> Result<(String, Vec<NewTicket>), Box<dyn std::error::Error + Send + Sync>>
where
    Fut: Future<Output = Result<(), StoreError>>,
{
    let OrderMessage::OrderPaid { data } = serde_json::from_slice(body)?;

    let order_id = data.order_id.clone();
    let tickets = issue_tickets(data, repository, upload_qr).await?;
    Ok((order_id, tickets))
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
    use chrono::Utc;

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

    #[test]
    fn generates_distinct_ticket_credentials() {
        use sha2::{Digest, Sha256};

        let first = super::generate_ticket_credential();
        let second = super::generate_ticket_credential();

        let expected: [u8; 32] = Sha256::digest(first.token.as_bytes()).into();

        assert_eq!(first.hash, expected);
        assert_ne!(first.hash, second.hash);
    }
}
