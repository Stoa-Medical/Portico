use portico_shared::Agent;
use prost_types::Struct;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Include the generated proto code
pub mod proto {
    tonic::include_proto!("portico");
}

// Export our modules
pub mod core;
pub mod handlers;

// Re-export important types
pub use crate::core::rpc_server::RpcServer;

// Thread-safe Agent map type
pub type SharedAgentMap = Arc<RwLock<HashMap<String, Agent>>>;

// Convert a protobuf Struct to a serde_json::Value
pub fn proto_struct_to_json(proto_struct: &Struct) -> Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &proto_struct.fields {
        map.insert(k.clone(), proto_value_to_json(v));
    }
    Value::Object(map)
}

// Convert a serde_json::Value to a protobuf Struct
pub fn json_to_proto_struct(json_value: &Value) -> prost_types::Struct {
    if let Value::Object(map) = json_value {
        let mut fields = std::collections::BTreeMap::new();
        for (k, v) in map {
            fields.insert(k.clone(), json_to_proto_value(v));
        }
        prost_types::Struct { fields }
    } else {
        prost_types::Struct {
            fields: std::collections::BTreeMap::new(),
        }
    }
}

// Convert a protobuf Value to a serde_json::Value
fn proto_value_to_json(proto_value: &prost_types::Value) -> Value {
    match &proto_value.kind {
        Some(prost_types::value::Kind::NullValue(_)) => Value::Null,
        Some(prost_types::value::Kind::NumberValue(n)) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Value::Number(num)
            } else {
                Value::Null
            }
        }
        Some(prost_types::value::Kind::StringValue(s)) => Value::String(s.clone()),
        Some(prost_types::value::Kind::BoolValue(b)) => Value::Bool(*b),
        Some(prost_types::value::Kind::StructValue(s)) => {
            let mut map = serde_json::Map::new();
            for (k, v) in &s.fields {
                map.insert(k.clone(), proto_value_to_json(v));
            }
            Value::Object(map)
        }
        Some(prost_types::value::Kind::ListValue(l)) => {
            Value::Array(l.values.iter().map(proto_value_to_json).collect())
        }
        None => Value::Null,
    }
}

// Convert a serde_json::Value to a protobuf Value
fn json_to_proto_value(json_value: &Value) -> prost_types::Value {
    let kind = match json_value {
        Value::Null => {
            prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into())
        }
        Value::Bool(b) => prost_types::value::Kind::BoolValue(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                prost_types::value::Kind::NumberValue(f)
            } else {
                prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into())
            }
        }
        Value::String(s) => prost_types::value::Kind::StringValue(s.clone()),
        Value::Array(a) => {
            let values = a.iter().map(json_to_proto_value).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        Value::Object(o) => {
            let mut fields = std::collections::BTreeMap::new();
            for (k, v) in o {
                fields.insert(k.clone(), json_to_proto_value(v));
            }
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };
    prost_types::Value { kind: Some(kind) }
}
