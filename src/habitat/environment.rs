/// Milieu physique d'un habitat à un instant donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Environment {
    Water,
    Land,
}
