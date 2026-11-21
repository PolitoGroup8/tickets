#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaidOrder {
    order_id: String,
    customer_id: String,
    lines: Vec<PaidOrderLine>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaidOrderLine {
    order_line_id: String,
    seat_id: String,
    seat_label: String,
}

#[derive(Debug, PartialEq, Eq)]
enum TicketStatus {
    Unscanned,
}

#[derive(Debug, PartialEq, Eq)]
struct NewTicket {
    order_line_id: String,
    seat_id: String,
    seat_label: String,
    status: TicketStatus,
}

fn issue_tickets(order: PaidOrder) -> Vec<NewTicket> {
    order
        .lines
        .into_iter()
        .map(|line| NewTicket {
            order_line_id: line.order_line_id,
            seat_id: line.seat_id,
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
            order_id: "ordine-1".into(),
            customer_id: "cliente-1".into(),
            lines: vec![
                PaidOrderLine {
                    order_line_id: "line-1".into(),
                    seat_id: "seat-14".into(),
                    seat_label: "E14".into(),
                },
                PaidOrderLine {
                    order_line_id: "line-2".into(),
                    seat_id: "seat-15".into(),
                    seat_label: "E15".into(),
                },
            ],
        };

        let tickets = issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        for (order_line_id,seat_id, seat_label) in [("line-1", "seat-14" ,"E14"), ("line-2", "seat-15", "E15")] {
            assert!(tickets.contains(&NewTicket {
                order_line_id: order_line_id.into(),
                seat_id: seat_id.into(),
                seat_label: seat_label.into(),
                status: TicketStatus::Unscanned
            }));
        }

        println!("{tickets:#?}");
    }


    #[test]
    fn reads_paid_order_lines_from_json(){
        let json = r#"{
            "orderId":"ordine-1",
            "customerId":"cliente-1",
            "lines":[
                { "orderLineId" : "line-1", "seatId" : "seat-14", "seatLabel" : "E14" },
                { "orderLineId" : "line-2", "seatId" : "seat-15", "seatLabel" : "E15" }
            ]
        }"#;

        let order: PaidOrder = serde_json::from_str(json).expect("valid order data");

        assert_eq!(order.lines.len(),2);
        assert_eq!(order.order_id, "ordine-1");
        assert_eq!(order.customer_id, "cliente-1");
        assert_eq!(order.lines[0].order_line_id, "line-1");
        assert_eq!(order.lines[0].seat_label, "E14");

        let tickets = issue_tickets(order);

        assert_eq!(tickets.len(), 2);
        println!("{tickets:#?}");

    }
}
