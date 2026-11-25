use std::sync::Arc;

use lapin::{Connection, ConnectionProperties, uri::AMQPUri};
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions},
    types::FieldTable,
};
use tokio_stream::StreamExt;

use crate::repository::{StoreError, TicketRepository};
use crate::store::PgTickets;

pub(crate) async fn connect() -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
    let mut uri = AMQPUri::default();

    uri.authority.host = std::env::var("RABBITMQ_HOST")?;
    uri.authority.port = std::env::var("RABBITMQ_PORT")?.parse()?;
    uri.authority.userinfo.username = std::env::var("RABBITMQ_DEFAULT_USER")?;
    uri.authority.userinfo.password = std::env::var("RABBITMQ_DEFAULT_PASS")?;

    Ok(Connection::connect_uri(uri, ConnectionProperties::default()).await?)
}

pub(crate) async fn publish_tickets_issued(
    channel: &lapin::Channel,
    event: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use lapin::{BasicProperties, Confirmation, options::BasicPublishOptions};

    let payload = serde_json::to_vec(event)?;
    let confirmation = channel
        .basic_publish(
            "tickets-events".into(),
            "tickets.issued".into(),
            BasicPublishOptions {
                mandatory: true,
                ..Default::default()
            },
            &payload,
            BasicProperties::default()
                .with_content_type("application/json".into())
                .with_delivery_mode(2),
        )
        .await?
        .await?;

    match confirmation {
        Confirmation::Ack(None) => Ok(()),
        Confirmation::Ack(Some(_)) => Err("TICKETS_ISSUED was not routed to a queue".into()),
        _ => Err("RabbitMQ did not confirm TICKETS_ISSUED".into()),
    }
}

async fn publish_pending_tickets_issued(
    channel: &lapin::Channel,
    repository: &PgTickets,
    order_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some((event_id, payload)) = repository.find_pending_tickets_issued(order_id).await? {
        publish_tickets_issued(channel, &payload).await?;
        repository.mark_tickets_issued_published(event_id).await?;
    }

    Ok(())
}

pub(crate) async fn consume_queue_message(
    connection: &Connection,
    repository: Arc<PgTickets>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let storage_client = crate::storage::client_from_env()?;
    let storage_bucket = std::env::var("RUSTFS_BUCKET")?;

    let client = &storage_client;
    let bucket = storage_bucket.as_str();

    let upload_qr = move |ticket_id: String, token| async move {
        let png = crate::qr::generate_ticket_qr_png(&ticket_id, token)
            .map_err(|err| StoreError::QrUploadFailed(err.to_string()))?;

        crate::storage::upload_ticket_qr(client, bucket, &ticket_id, png)
            .await
            .map_err(|err| StoreError::QrUploadFailed(err.to_string()))?;

        Ok(())
    };

    let queue = "tickets.order-paid";
    let channel = connection.create_channel().await?;

    channel
        .queue_declare(
            queue.into(),
            QueueDeclareOptions::durable(),
            FieldTable::default(),
        )
        .await?;

    let publisher = connection.create_channel().await?;
    publisher
        .confirm_select(lapin::options::ConfirmSelectOptions::default())
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

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;

        // STORAGE NEL DB

        match crate::service::issue_tickets_from_message(&delivery.data, &repository, &upload_qr)
            .await
        {
            Ok((order_id, tickets)) => {
                println!("{tickets:#?}");

                if let Err(err) =
                    publish_pending_tickets_issued(&publisher, &repository, &order_id).await
                {
                    delivery
                        .reject(BasicRejectOptions { requeue: true })
                        .await?;
                    return Err(err);
                }

                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(err) => {
                let requeue = matches!(
                    err.downcast_ref::<StoreError>(),
                    Some(StoreError::QrUploadFailed(_))
                );

                delivery.reject(BasicRejectOptions { requeue }).await?;

                if requeue {
                    return Err(err);
                }

                eprintln!("Cannot process order message: {err}");
            }
        }

        // I TICKET GENERATI LI RIMANDIAMO SULLA CODA
    }

    Ok(())
}
