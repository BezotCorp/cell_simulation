use crate::definitions::FounderDefinition;
use crate::ecosystem::{KillTarget, ResourceKind};
use serde::{Deserialize, Serialize};

/// Action de l'expérimentateur sur l'expérience en cours. Elle s'applique entre deux pas de
/// simulation et laisse l'état cohérent (identités, filiation, liaisons, sauvegardes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Intervention {
    /// Crée un groupe de cellules, au site `at` ou selon la règle du groupe.
    Spawn { group: FounderDefinition, at: Option<(usize, usize)> },
    Kill(KillTarget),
    /// Charge `sites` sites d'électricité statique à `intensity` fois le seuil de claquage.
    Storm { sites: u32, intensity: f32 },
    /// Forme des nuages de `amount` d'eau équivalente sur `sites` sites au hasard.
    Rain { sites: u32, amount: f32 },
    /// Pose `count` éléments de `amount` (énergie, intensité, profondeur…) sur des sites libres.
    Scatter { resource: ResourceKind, count: u32, amount: f32 },
    /// Remplit toutes les réserves alimentaires.
    FillFood,
}
