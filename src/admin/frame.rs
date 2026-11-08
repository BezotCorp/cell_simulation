use serde::{Serialize, de::DeserializeOwned};

/// Codage des échanges : un message RON sur une ligne.
pub(crate) struct Frame;

impl Frame {
    pub(crate) fn encode<T: Serialize>(message: &T) -> Result<String, String> {
        ron::ser::to_string(message).map_err(|e| e.to_string())
    }

    pub(crate) fn decode<T: DeserializeOwned>(line: &str) -> Result<T, String> {
        ron::from_str(line).map_err(|e| e.to_string())
    }
}
