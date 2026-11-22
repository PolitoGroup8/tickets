use async_trait::async_trait;
use sqlx::PgPool;




pub struct PgTickets {
    pool: PgPool
}

