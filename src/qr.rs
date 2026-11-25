use std::io::Cursor;

use image::{DynamicImage, ImageFormat, Luma};
use qrcode::QrCode;
use uuid::Uuid;

pub(crate) fn generate_ticket_qr_png(
    ticket_id: &str,
    token: Uuid,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let payload = format!("{ticket_id}.{token}");
    let qr = QrCode::new(payload.as_bytes())?;
    let pixels = qr.render::<Luma<u8>>().build();

    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageLuma8(pixels).write_to(&mut png, ImageFormat::Png)?;
    Ok(png.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_ticket_qr_png() {
        let ticket_id = Uuid::new_v4().to_string();
        let png = generate_ticket_qr_png(&ticket_id, Uuid::new_v4()).unwrap();

        let decoded = image::load_from_memory_with_format(&png, ImageFormat::Png).unwrap();
        assert_eq!(decoded.width(), decoded.height());
    }
}