#![forbid(unsafe_code)]

pub mod web_read;
use act_interfaces::{
    YoutubeAction, YoutubeControlError, YoutubeControlFailure, YoutubeControlRequest,
    YoutubeControlResponse, YoutubeControlSuccess,
};
use async_trait::async_trait;
use ores_lib_core::{redact_value, valid_correlation_id};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use serde_json::Value;
use thiserror::Error;

const MAX_IDEMPOTENCY_KEY_BYTES: usize = 200;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("database capability must be read-only")]
    WriteCapableDatabase,
    #[error("invalid command request")]
    InvalidRequest,
    #[error("invalid request identifier")]
    InvalidRequestId,
    #[error("stored command result is invalid")]
    InvalidStoredResult,
    #[error("database query failed")]
    Database(#[from] DbErr),
}

pub fn validate_request(request: &YoutubeControlRequest) -> Result<(), CoreError> {
    let encoded_payload =
        serde_json::to_vec(&request.payload).map_err(|_| CoreError::InvalidRequest)?;
    if encoded_payload.len() > MAX_PAYLOAD_BYTES {
        return Err(CoreError::InvalidRequest);
    }

    let key_is_valid = request.idempotency_key.as_deref().is_none_or(|key| {
        !key.is_empty()
            && key.len() <= MAX_IDEMPOTENCY_KEY_BYTES
            && key.bytes().all(|byte| byte.is_ascii_graphic())
    });
    if !key_is_valid || (is_mutating(request.action) && request.idempotency_key.is_none()) {
        return Err(CoreError::InvalidRequest);
    }

    Ok(())
}

pub fn safe_request_id(value: &str) -> Result<&str, CoreError> {
    valid_correlation_id(value)
        .then_some(value)
        .ok_or(CoreError::InvalidRequestId)
}

pub fn telemetry_field<'a>(key: &str, value: &'a str) -> &'a str {
    redact_value(key, value)
}

fn is_mutating(action: YoutubeAction) -> bool {
    matches!(
        action,
        YoutubeAction::ExportAnalytics
            | YoutubeAction::StartUpload
            | YoutubeAction::ProcessUpload
            | YoutubeAction::ProcessAllUploads
            | YoutubeAction::PublishVideo
            | YoutubeAction::UpdateVideo
            | YoutubeAction::CreatePlaylist
            | YoutubeAction::AddToPlaylist
            | YoutubeAction::IngestGmail
            | YoutubeAction::SendDigest
    )
}

#[async_trait]
pub trait YoutubeCommandResultStore: Send + Sync {
    async fn fetch_public_result(
        &self,
        request_id: &str,
    ) -> Result<Option<YoutubeControlResponse>, CoreError>;
}

pub struct SeaOrmYoutubeCommandResultStore {
    connection: DatabaseConnection,
}

impl SeaOrmYoutubeCommandResultStore {
    pub fn new(connection: DatabaseConnection, access: DatabaseAccess) -> Result<Self, CoreError> {
        if access != DatabaseAccess::ReadOnly {
            return Err(CoreError::WriteCapableDatabase);
        }
        Ok(Self { connection })
    }
}

#[async_trait]
impl YoutubeCommandResultStore for SeaOrmYoutubeCommandResultStore {
    async fn fetch_public_result(
        &self,
        request_id: &str,
    ) -> Result<Option<YoutubeControlResponse>, CoreError> {
        let request_id = safe_request_id(request_id)?;
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT ok, request_id, duration_ms, data::text AS data_json, \
                    error_code, error_message, error_details::text AS error_details_json \
             FROM public_youtube_command_results \
             WHERE request_id = $1 AND is_public = TRUE LIMIT 1",
            [request_id.into()],
        );

        let Some(row) = self.connection.query_one(statement).await? else {
            return Ok(None);
        };

        let ok: bool = row.try_get("", "ok")?;
        let stored_request_id: String = row.try_get("", "request_id")?;
        safe_request_id(&stored_request_id)?;

        if ok {
            let duration_ms: i64 = row.try_get("", "duration_ms")?;
            let duration_ms = duration_ms
                .try_into()
                .map_err(|_| CoreError::InvalidStoredResult)?;
            let data_json: String = row.try_get("", "data_json")?;
            let data: Value =
                serde_json::from_str(&data_json).map_err(|_| CoreError::InvalidStoredResult)?;
            return Ok(Some(YoutubeControlResponse::Success(
                YoutubeControlSuccess {
                    ok: true,
                    request_id: stored_request_id,
                    duration_ms,
                    data,
                },
            )));
        }

        let code: String = row.try_get("", "error_code")?;
        let message: String = row.try_get("", "error_message")?;
        if code.is_empty() || message.is_empty() {
            return Err(CoreError::InvalidStoredResult);
        }
        let details_json: Option<String> = row.try_get("", "error_details_json")?;
        let details = details_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|_| CoreError::InvalidStoredResult)?;
        Ok(Some(YoutubeControlResponse::Failure(
            YoutubeControlFailure {
                ok: false,
                request_id: Some(stored_request_id),
                error: YoutubeControlError {
                    code,
                    message,
                    details,
                },
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use serde_json::Map;

    use super::*;

    fn request(action: YoutubeAction, idempotency_key: Option<&str>) -> YoutubeControlRequest {
        YoutubeControlRequest {
            action,
            payload: Map::new(),
            idempotency_key: idempotency_key.map(str::to_owned),
        }
    }

    #[test]
    fn mutating_commands_require_bounded_idempotency() {
        assert!(validate_request(&request(YoutubeAction::PublishVideo, None)).is_err());
        assert!(
            validate_request(&request(YoutubeAction::PublishVideo, Some("publish-123"))).is_ok()
        );
        assert!(validate_request(&request(
            YoutubeAction::PublishVideo,
            Some(&"x".repeat(MAX_IDEMPOTENCY_KEY_BYTES + 1)),
        ))
        .is_err());
        assert!(validate_request(&request(YoutubeAction::Channel, None)).is_ok());
    }

    #[test]
    fn payloads_are_bounded() {
        let mut payload = Map::new();
        payload.insert(
            "oversized".into(),
            serde_json::Value::String(iter::repeat_n('x', MAX_PAYLOAD_BYTES).collect()),
        );
        let command = YoutubeControlRequest {
            action: YoutubeAction::Channel,
            payload,
            idempotency_key: None,
        };

        assert!(validate_request(&command).is_err());
    }

    #[test]
    fn telemetry_uses_the_ores_redaction_contract() {
        assert_eq!(
            telemetry_field("authorization", "Bearer example"),
            "[REDACTED]"
        );
        assert_eq!(
            safe_request_id("request-12345678").unwrap(),
            "request-12345678"
        );
        assert!(safe_request_id("bad id").is_err());
    }
}
