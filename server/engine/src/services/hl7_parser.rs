use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// Parse an HL7v2 message, extracting fields from the MSH (Message Header) segment.
///
/// This is a basic implementation that handles pipe-delimited MSH parsing only.
/// Future versions will parse additional segments (PID, PV1, OBR, OBX, etc.).
///
/// MSH segment layout (pipe-delimited, field separator is |):
///   MSH|^~\&|SendingApp|SendingFacility|ReceivingApp|ReceivingFacility|DateTime|Security|MessageType|ControlID|ProcessingID|VersionID
///
/// Returns a JSON object with extracted MSH fields.
pub fn parse_hl7v2(raw: &str) -> Result<Value> {
    // Find the MSH segment (should be the first line)
    let msh_line = raw
        .lines()
        .find(|line| line.starts_with("MSH"))
        .ok_or_else(|| anyhow!("No MSH segment found in HL7v2 message"))?;

    // The field separator is the character immediately after "MSH"
    // By convention this is '|', but we read it from the message
    let field_sep = msh_line
        .chars()
        .nth(3)
        .ok_or_else(|| anyhow!("MSH segment too short to contain field separator"))?;

    // Split into fields. MSH-1 is the field separator itself,
    // so the split gives us: ["MSH", "^~\\&", "SendingApp", ...]
    // where index 0 = "MSH", index 1 = encoding chars (MSH-2), etc.
    let fields: Vec<&str> = msh_line.split(field_sep).collect();

    // Helper to safely get a field by MSH field number (1-indexed).
    // MSH-1 is the field separator (index 0 in split is "MSH").
    // MSH-2 is encoding characters (index 1).
    // MSH-3 is sending application (index 2), etc.
    let get_field = |msh_idx: usize| -> Option<&str> {
        if msh_idx < 2 {
            return None;
        }
        fields.get(msh_idx - 1).copied()
    };

    // Extract message type (MSH-9), which may contain components like "ADT^A01"
    let message_type_raw = get_field(9).unwrap_or("");
    let message_type_components: Vec<&str> = message_type_raw.split('^').collect();

    let message_type = json!({
        "raw": message_type_raw,
        "message_code": message_type_components.first().copied().unwrap_or(""),
        "trigger_event": message_type_components.get(1).copied().unwrap_or(""),
        "message_structure": message_type_components.get(2).copied().unwrap_or(""),
    });

    Ok(json!({
        "field_separator": field_sep.to_string(),
        "encoding_characters": get_field(2).unwrap_or(""),
        "sending_application": get_field(3).unwrap_or(""),
        "sending_facility": get_field(4).unwrap_or(""),
        "receiving_application": get_field(5).unwrap_or(""),
        "receiving_facility": get_field(6).unwrap_or(""),
        "date_time": get_field(7).unwrap_or(""),
        "security": get_field(8).unwrap_or(""),
        "message_type": message_type,
        "message_control_id": get_field(10).unwrap_or(""),
        "processing_id": get_field(11).unwrap_or(""),
        "version_id": get_field(12).unwrap_or(""),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_msh_segment() {
        let raw = "MSH|^~\\&|SendingApp|SendingFac|RecvApp|RecvFac|20240101120000||ADT^A01^ADT_A01|MSG00001|P|2.5.1";
        let result = parse_hl7v2(raw).unwrap();

        assert_eq!(result["sending_application"], "SendingApp");
        assert_eq!(result["sending_facility"], "SendingFac");
        assert_eq!(result["message_type"]["message_code"], "ADT");
        assert_eq!(result["message_type"]["trigger_event"], "A01");
        assert_eq!(result["message_control_id"], "MSG00001");
        assert_eq!(result["version_id"], "2.5.1");
    }

    #[test]
    fn test_no_msh_segment() {
        let raw = "PID|1||12345^^^MRN||Doe^John";
        assert!(parse_hl7v2(raw).is_err());
    }
}
