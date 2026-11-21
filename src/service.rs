struct PaidOrder {
    lines: Vec<PaidOrderLine>,
}

struct PaidOrderLine {
    order_line_id: String,
    seat_label: String,
}

#[derive(Debug, PartialEq, Eq)]
enum TicketStatus {
    Unscanned,
}

#[derive(Debug, PartialEq, Eq)]
struct NewTicket {
    order_line_id: String,
    seat_label: String,
    status: TicketStatus,
}

fn issue_tickets(order: PaidOrder) -> Vec<NewTicket> {
    order
        .lines
        .into_iter()
        .map(|line| NewTicket {
            order_line_id: line.order_line_id,
            seat_label: line.seat_label,
            status: TicketStatus::Unscanned,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_one_unscanned_ticket_per_order_line() {
        let order = PaidOrder {
            lines: vec![
                PaidOrderLine {
                    order_line_id: "line-1".into(),
                    seat_label: "E14".into(),
                },
                PaidOrderLine {
                    order_line_id: "line-2".into(),
                    seat_label: "E15".into(),
                },
            ],
        };

        let tickets = issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        for (order_line_id, seat_label) in [("line-1", "E14"), ("line-2", "E15")] {
            assert!(tickets.contains(&NewTicket {
                order_line_id: order_line_id.into(),
                seat_label: seat_label.into(),
                status: TicketStatus::Unscanned
            }));
        }

        println!("{tickets:#?}");
    }
}
