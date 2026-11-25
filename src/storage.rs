use std::env;

use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
};

use aws_sdk_s3::primitives::ByteStream;

pub(crate) fn client_from_env() -> Result<Client, env::VarError> {
    let credentials = Credentials::new(
        env::var("RUSTFS_ACCESS_KEY")?,
        env::var("RUSTFS_SECRET_KEY")?,
        None,
        None,
        "rustfs-env",
    );

    let config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .endpoint_url(env::var("RUSTFS_ENDPOINT")?)
        .region(Region::new("us-east-1"))
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    Ok(Client::from_conf(config))
}

pub(crate) fn ticket_qr_key(ticket_id: &str) -> String {
    format!("tickets/{ticket_id}/qr.png")
}

pub(crate) async fn upload_ticket_qr(
    client: &Client,
    bucket: &str,
    ticket_id: &str,
    png: Vec<u8>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let key = ticket_qr_key(ticket_id);

    client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .content_type("image/png")
        .body(ByteStream::from(png))
        .send()
        .await?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires local RustFS"]
    async fn reaches_ticket_qr_bucket() {
        let _ = dotenvy::dotenv();
        let client = client_from_env().expect("RustFS configuration");
        let bucket = env::var("RUSTFS_BUCKET").expect("RUSTFS_BUCKET");

        client
            .head_bucket()
            .bucket(bucket)
            .send()
            .await
            .expect("accessible RustFS bucket");
    }

    #[tokio::test]
    #[ignore = "requires local RustFS"]
    async fn uploads_ticket_qr() {
        let _ = dotenvy::dotenv();
        let client = client_from_env().unwrap();
        let bucket = env::var("RUSTFS_BUCKET").unwrap();
        let ticket_id = uuid::Uuid::new_v4().to_string();
        let png = crate::qr::generate_ticket_qr_png(&ticket_id, uuid::Uuid::new_v4()).unwrap();
        let expected_size = i64::try_from(png.len()).unwrap();

        let key = upload_ticket_qr(&client, &bucket, &ticket_id, png)
            .await
            .unwrap();
        let object = client
            .head_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .unwrap();

        client
            .delete_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .unwrap();

        assert_eq!(key, format!("tickets/{ticket_id}/qr.png"));
        assert_eq!(object.content_type(), Some("image/png"));
        assert_eq!(object.content_length(), Some(expected_size));
    }
}
