use crate::models::agents::Agent;
use crate::models::runtime_sessions::RuntimeSession;
use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SignalType {
    Run,
    Sync,
    Fyi,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::Run => "run",
            SignalType::Sync => "sync",
            SignalType::Fyi => "fyi",
        }
    }
}

impl FromStr for SignalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "run" => Ok(SignalType::Run),
            "command" => Ok(SignalType::Run), // For backward compatibility
            "sync" => Ok(SignalType::Sync),
            "fyi" => Ok(SignalType::Fyi),
            _ => Err(format!("Invalid signal type: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for SignalType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("signal_type")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SignalType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.as_str()? {
            "run" => Ok(SignalType::Run),
            "command" => Ok(SignalType::Run), // For backward compatibility
            "sync" => Ok(SignalType::Sync),
            "fyi" => Ok(SignalType::Fyi),
            _ => Err("Invalid signal type".into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SignalType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.as_str();
        buf.extend_from_slice(s.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunPayload {
    pub operation: String,
    pub payload: RunDataPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunDataPayload {
    pub id: String,
    pub properties: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub scope: String,
    pub mode: String,
    pub targets: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Signal {
    pub identifiers: IdFields<i64>,
    pub timestamps: TimestampFields,
    pub user_requested_uuid: String,
    pub agent: Option<Agent>,
    pub linked_rts: Option<RuntimeSession>,
    pub signal_type: SignalType,
    pub initial_data: Option<Value>,
    pub result_data: Option<Value>,
    pub error_message: Option<String>,
}
