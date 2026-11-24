use std::sync::Arc;

use lapin::message::Delivery;
use lapin::protocol::connection;
use lapin::{Connection, ConnectionProperties, uri::AMQPUri};
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions},
    types::FieldTable,
};
use tokio_stream::StreamExt;

use crate::store::PgTickets;

pub(crate) async fn connect() -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
    let mut uri = AMQPUri::default();

    uri.authority.host = std::env::var("RABBITMQ_HOST")?;
    uri.authority.port = std::env::var("RABBITMQ_PORT")?.parse()?;
    uri.authority.userinfo.username = std::env::var("RABBITMQ_DEFAULT_USER")?;
    uri.authority.userinfo.password = std::env::var("RABBITMQ_DEFAULT_PASS")?;

    Ok(Connection::connect_uri(uri, ConnectionProperties::default()).await?)
}

pub(crate) async fn consume_queue_message(connection: &Connection, repository: Arc<PgTickets>) -> lapin::Result<()> {
    let queue = "tickets.order-paid";
    let channel = connection.create_channel().await?;

    channel
        .queue_declare(
            queue.into(),
            QueueDeclareOptions::durable(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            queue.into(),
            "tickets_service".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("Listening on {queue}");

    while let Some(delivery) = consumer.next().await{
        let delivery = delivery?;

        // STORAGE NEL DB

        match crate::service::issue_tickets_from_message(&delivery.data, &repository).await{
            Ok(tickets) => {
                println!("{tickets:#?}");
                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(err)=> {
                eprintln!("Invalid order message: {err}");
                delivery.reject(BasicRejectOptions{requeue:false}).await?;
            }
        }

        // I TICKET GENERATI LI RIMANDIAMO SULLA CODA
    }

    Ok(())
}
