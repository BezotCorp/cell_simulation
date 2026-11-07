#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) enum Mineral {
    Iron,
    Calcium,
    Phosphorus,
}
