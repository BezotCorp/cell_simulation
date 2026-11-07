/// Grandeur biologique ou physique représentée par un dégradé de couleurs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Gradient {
    Energy,
    Temperature,
    Adaptation,
}
