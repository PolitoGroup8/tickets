//! The single mapping point: every failure of the service becomes an RFC 9457
//! problem document here, and nowhere else.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use validator::{Validate, ValidationErrors};

use crate::repository::StoreError;


/// A failure, classified by the status class of the policy table.
pub enum AppError {
    /// The request cannot be read: malformed body, unknown or missing field, wrong type.
    Malformed(String),
    /// The URL names nothing.
    NotFound(String),
    /// The request is valid, but the state of the resource does not allow it.
    Conflict(String),
    /// The request is well formed, but a value cannot be processed.
    Invalid(String),
    /// Some fields of the body are not valid: a 422 with one entry of `errors` for each.
    InvalidFields(Vec<FieldError>),
    /// Any other status, for example 405 or 415: the problem type stays `about:blank`.
    Other(StatusCode, String),
}

/// The database refused a statement, or could not be reached: a 500, with the cause in the log
/// only.
impl From<StoreError> for AppError {
    fn from(e: StoreError) -> AppError {
        tracing::error!("the database could not complete the request: {e:?}");
        AppError::Other(StatusCode::INTERNAL_SERVER_ERROR, "the database could not complete the request".to_string())
    }
}

/// One entry of the member `errors` of a 422 (L16, slide 12).
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub detail: String,
}

impl AppError {
    /// A value of the field `field` that a type refused, with the reason given by the type.
    pub fn field(field: &str, detail: String) -> AppError {
        AppError::InvalidFields(vec![FieldError {
            field: field.to_string(),
            code: "invalid_format".to_string(),
            detail,
        }])
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::InvalidFields(mut errors) = self {
            errors.sort_by(|a, b| (&a.field, &a.code).cmp(&(&b.field, &b.code)));
            let mut fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
            fields.dedup();
            let body = json!({
                // TODO type
                "type": "https://tickets.example.com/problems/validation",
                "title": "Validation failed",
                "status": 422,
                "detail": format!("invalid fields: {}", fields.join(", ")),
                "errors": errors,
            });
            let status = StatusCode::UNPROCESSABLE_ENTITY;
            return (status, [(header::CONTENT_TYPE, "application/problem+json")], body.to_string()).into_response();
        }
        let (status, slug, detail) = match self {
            AppError::Malformed(d) => (StatusCode::BAD_REQUEST, Some(("malformed-request", "Malformed request")), d),
            AppError::NotFound(d) => (StatusCode::NOT_FOUND, Some(("not-found", "Not found")), d),
            AppError::Conflict(d) => (StatusCode::CONFLICT, Some(("conflict", "Conflict")), d),
            AppError::Invalid(d) => (StatusCode::UNPROCESSABLE_ENTITY, Some(("validation", "Validation failed")), d),
            AppError::Other(status, d) => (status, None, d),
            AppError::InvalidFields(_) => unreachable!("answered above"),
        };
        let body = match slug {
            Some((slug, title)) => json!({
                // TODO type
                "type": format!("https://tickets.example.com/problems/{slug}"),
                "title": title,
                "status": status.as_u16(),
                "detail": detail,
            }),
            None => json!({
                "title": status.canonical_reason().unwrap_or("Error"),
                "status": status.as_u16(),
                "detail": detail,
            }),
        };
        (status, [(header::CONTENT_TYPE, "application/problem+json")], body.to_string()).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> AppError {
        match rejection {
            JsonRejection::MissingJsonContentType(r) => {
                AppError::Other(StatusCode::UNSUPPORTED_MEDIA_TYPE, r.body_text())
            }
            other => AppError::Malformed(other.body_text()),
        }
    }
}

/// Every violation of the `#[validate(...)]` rules of a body, with the code that the rule declares.
impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> AppError {
        let mut list = Vec::new();
        for (field, violations) in errors.field_errors() {
            for violation in violations {
                list.push(FieldError {
                    field: field.to_string(),
                    code: violation.code.to_string(),
                    detail: violation.message.as_deref().unwrap_or("is not valid").to_string(),
                });
            }
        }
        AppError::InvalidFields(list)
    }
}

/// A JSON body that has passed its `#[validate(...)]` rules (L16, slides 15-16). Its rejection
/// is an `AppError`: a body that cannot be read is a 400, as with `Json`, and a body that breaks
/// some rules is a 422 that lists all of them.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> AppError {
        AppError::Malformed(rejection.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> AppError {
        AppError::Malformed(rejection.body_text())
    }
}

/// The answer to a path that matches no route.
pub async fn no_route() -> AppError {
    AppError::NotFound("no resource at this path".to_string())
}

/// The answer to a known path called with a method it does not support.
pub async fn wrong_method() -> AppError {
    AppError::Other(StatusCode::METHOD_NOT_ALLOWED, "method not supported on this path".to_string())
}
