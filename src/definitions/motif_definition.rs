use crate::genetics::Orientation;
use serde::{Deserialize, Serialize};

/// Un motif binaire du catalogue : une courte suite de bits (1 à 8) cherchée dans chaque séquence de
/// 8 bits du génome, et le sens dans lequel on la lit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MotifDefinition {
    pub(crate) bits: String,
    #[serde(default)]
    pub(crate) orientation: Orientation,
}

impl MotifDefinition {
    pub(crate) fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Motif sous forme de valeur et de longueur : le premier bit écrit est le bit de poids fort.
    pub(crate) fn pattern(&self) -> Result<(u8, u8), String> {
        let length = self.bits.chars().count();
        if !(1..=8).contains(&length) || self.bits.chars().any(|c| c != '0' && c != '1') {
            return Err(format!("Motif invalide : « {} » (1 à 8 bits, 0 et 1).", self.bits));
        }
        let value = self.bits.chars().fold(0u8, |value, bit| (value << 1) | u8::from(bit == '1'));
        Ok((value, length as u8))
    }
}
