use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum ViewMode {
    Ecosystem,
    Temperature,
    Adaptation,
    /// Colore chaque organisme selon la façon dont il obtient son énergie.
    Strategy,
}
