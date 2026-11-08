use crate::genetics::Orientation;

/// Motif binaire compilé : `length` bits (1 à 8) de valeur `pattern`, cherché dans chaque séquence
/// de 8 bits selon son orientation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Motif {
    pattern: u8,
    length: u8,
    orientation: Orientation,
}

impl Motif {
    pub(crate) fn new(pattern: u8, length: u8, orientation: Orientation) -> Self {
        Self {
            pattern,
            length,
            orientation,
        }
    }

    /// Inverse l'ordre des `length` bits de `window`.
    fn reversed(&self, window: u8) -> u8 {
        window.reverse_bits() >> (8 - self.length)
    }

    /// Le motif se trouve-t-il dans la fenêtre de `length` bits qui commence au bit `start`
    /// (0 = bit de poids fort) de `byte` ? Plusieurs positions voisines peuvent correspondre :
    /// les occurrences se chevauchent.
    pub(crate) fn matches_at(&self, byte: u8, start: u8) -> bool {
        let mask = if self.length == 8 { 0xFF } else { (1u8 << self.length) - 1 };
        let window = (byte >> (8 - start - self.length)) & mask;
        match self.orientation {
            Orientation::Forward => window == self.pattern,
            Orientation::Reverse => self.reversed(window) == self.pattern,
            Orientation::Both => window == self.pattern || self.reversed(window) == self.pattern,
        }
    }

    pub(crate) fn length(&self) -> u8 {
        self.length
    }
}
