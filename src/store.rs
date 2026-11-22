use async_trait::async_trait;
use sqlx::PgPool;

use crate::repository::TicketsRepository;



pub struct PgTickets {
    pool: PgPool
}

#[async_trait]
impl TicketsRepository for PgTickets {
    
}

