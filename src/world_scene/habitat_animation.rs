/// Animation continue d'une ressource ; seules les ressources animées coûtent du travail par image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HabitatAnimation {
    /// Immobile.
    Still,
    /// Respiration lumineuse d'une source thermique.
    Pulse,
    /// Bulles qui montent et descendent doucement.
    Bob,
}
