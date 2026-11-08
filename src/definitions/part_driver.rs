use crate::genetics::Product;
use serde::{Deserialize, Serialize};

/// Ce qui développe un composant visible : son niveau (de 0 à 1) vient d'ici.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum PartDriver {
    /// Niveau du produit exprimé par le génome.
    Product(Product),
    /// Unités de tissu nerveux rapportées à huit.
    NeuralUnits,
    /// Toujours présent, à plein développement.
    Always,
}
