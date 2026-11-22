use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction, prelude::FromRow};
use uuid::Uuid;

use crate::repository::{
    Order, OrderLine, StoreError, TicketRecord, TicketRepository, TicketTransaction,
};

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> StoreError {
        StoreError::Failed(e.to_string())
    }
}

struct PgTransaction {
    tx: Transaction<'static, Postgres>,
    meta_id: Option<Uuid>,
}

#[derive(FromRow)]
struct TicketRow {
    ticket_id: Uuid,
    meta_id: Uuid,
    order_line_id: String,
    sector_id: String,
    sector_name: String,
    seat_id: String,
    seat_label: String,
    seat_row: Option<String>,
    seat_number: Option<i32>,
    issued_at: String,
    status: String,
}

impl From<TicketRow> for TicketRecord {
    fn from(r: TicketRow) -> TicketRecord {
        TicketRecord {
            ticket_id: r.ticket_id.to_string(),
            meta_id: r.meta_id.to_string(),
            order_line_id: r.order_line_id,
            sector_id: r.sector_id,
            sector_name: r.sector_name,
            seat_id: r.seat_id,
            seat_label: r.seat_label,
            seat_row: r.seat_row,
            seat_number: r.seat_number,
            issued_at: r.issued_at,
            status: r.status,
        }
    }
}

struct PgTickets {
    pool: PgPool,
}

impl PgTickets {
    fn new(pool: PgPool) -> PgTickets {
        PgTickets { pool }
    }
}

#[async_trait]
impl TicketRepository for PgTickets {
    async fn begin(&self) -> Result<Box<dyn TicketTransaction>, StoreError> {
        let tx = self.pool.begin().await?;

        Ok(Box::new(PgTransaction { tx, meta_id: None }))
    }

    async fn find_by_order_id(
        &self,
        order_id: &str,
    ) -> Result<Option<Vec<TicketRecord>>, StoreError> {
        let rows = sqlx::query_as::<_, TicketRow>(
            r#"SELECT t.ticket_id, t.meta_id, t.order_line_id,
                      t.sector_id, t.sector_name, t.seat_id,
                      t.seat_label, t.seat_row, t.seat_number,
                      to_char(t.issued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS issued_at,
                      t.status
               FROM ticket t
               JOIN ticket_meta tm ON tm.ticket_meta_id = t.meta_id
               WHERE tm.order_id = $1"#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        Ok(Some(rows.into_iter().map(TicketRecord::from).collect()))
    }

    async fn find_by_ticket_id(&self, ticket_id: &str) -> Result<Option<TicketRecord>, StoreError> {
        let row = sqlx::query_as::<_, TicketRow>(
            r#"SELECT ticket_id, meta_id, order_line_id,
                      sector_id, sector_name, seat_id,
                      seat_label, seat_row, seat_number,
                      to_char(issued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS issued_at,
                      status
               FROM ticket
               WHERE ticket_id = $1::UUID"#,
        )
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(TicketRecord::from))
    }
}

#[async_trait]
impl TicketTransaction for PgTransaction {
    async fn save_order(&mut self, order: &Order) -> Result<(), StoreError> {
        let meta_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO ticket_meta (
                order_id,
                event_id,
                event_title,
                session_id,
                session_starts_at,
                venue_name,
                venue_city,
                customer_id,
                customer_email,
                customer_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING ticket_meta_id
            "#,
        )
        .bind(&order.order_id)
        .bind(&order.event_id)
        .bind(&order.event_title)
        .bind(&order.session_id)
        .bind(order.session_starts_at)
        .bind(&order.venue_name)
        .bind(&order.venue_city)
        .bind(&order.customer_id)
        .bind(&order.customer_email)
        .bind(&order.customer_name)
        .fetch_one(&mut *self.tx)
        .await?;

        self.meta_id = Some(meta_id);

        Ok(())
    }

    async fn save_ticket(&mut self, order_line: &OrderLine) -> Result<(), StoreError> {
        let meta_id = self
            .meta_id
            .ok_or_else(|| StoreError::Failed("Order not saved".into()))?;

        sqlx::query(
            r#"
            INSERT INTO ticket (
                meta_id,
                order_line_id,
                sector_id,
                sector_name,
                seat_id,
                seat_label
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(meta_id)
        .bind(&order_line.order_line_id)
        .bind(&order_line.sector_id)
        .bind(&order_line.sector_name)
        .bind(&order_line.seat_id)
        .bind(&order_line.seat_label)
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), StoreError> {
        self.tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_save_and_retrieve_tickets() {
        let pool = PgPool::connect(
            "postgres://user:password@localhost:5432/TICKETS"
        )
        .await
        .unwrap();

        let repository = PgTickets::new(pool);

        let order = Order {
            order_id: "test-order-001".into(),
            customer_id: "customer-001".into(),
            event_id: "event-001".into(),
            event_title: "Test Concert".into(),
            session_id: "session-001".into(),
            session_starts_at: Utc.with_ymd_and_hms(2026, 12, 10, 20, 0, 0).unwrap(),
            venue_name: "Test Arena".into(),
            venue_city: "Milan".into(),
            customer_email: "test@example.com".into(),
            customer_name: Some("Test User".into()),
            lines: vec![
                OrderLine {
                    order_line_id: "line-001".into(),
                    seat_id: "seat-001".into(),
                    seat_label: "A1".into(),
                    sector_id: "sector-001".into(),
                    sector_name: "VIP".into(),
                },
            ],
        };

        let mut tx = repository.begin().await.unwrap();

        tx.save_order(&order).await.unwrap();

        for line in &order.lines {
            tx.save_ticket(line).await.unwrap();
        }

        tx.commit().await.unwrap();

        let tickets = repository
            .find_by_order_id(&order.order_id)
            .await
            .unwrap()
            .expect("Tickets not found");

        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].seat_label, "A1");

        let ticket = repository
            .find_by_ticket_id(&tickets[0].ticket_id)
            .await
            .unwrap();

        assert!(ticket.is_some());
    }
}
