use crate::definitions::{Appearance, MaterialKind, SpeciesDefinition, WorldDefinition};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, OnceLock},
};
use uuid::{Uuid, Version};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Definitions {
    id: Uuid,
    revision: u32,
    species: Arc<SpeciesDefinition>,
    world: Arc<WorldDefinition>,
    appearance: Appearance,
}

impl Definitions {
    pub(crate) fn load(path: impl AsRef<Path>) -> Result<Arc<Self>, String> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("{} : {e}", path.as_ref().display()))?;
        let definitions: Self =
            ron::from_str(&text).map_err(|e| format!("Définitions RON : {e}"))?;
        definitions.validate()?;
        Ok(Arc::new(definitions))
    }

    pub(crate) fn builtin() -> Arc<Self> {
        static DEFINITIONS: OnceLock<Arc<Definitions>> = OnceLock::new();
        DEFINITIONS
            .get_or_init(|| {
                let data: Definitions = ron::from_str(include_str!("../../data/definitions.ron"))
                    .expect("catalogue embarqué valide");
                data.validate().expect("invariants du catalogue embarqué");
                Arc::new(data)
            })
            .clone()
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn revision(&self) -> u32 {
        self.revision
    }

    pub(crate) fn default_seed(&self) -> u64 {
        self.world.seed()
    }

    /// Espèce modifiable : les règles compilées sont invalidées, elles se reconstruiront à l'usage.
    pub(crate) fn species_mut(&mut self) -> &mut SpeciesDefinition {
        Arc::make_mut(&mut self.species).edit()
    }

    pub(crate) fn world_mut(&mut self) -> &mut WorldDefinition {
        Arc::make_mut(&mut self.world)
    }

    pub(crate) fn appearance_mut(&mut self) -> &mut Appearance {
        &mut self.appearance
    }

    /// Nouvelle révision du catalogue, à chaque enregistrement.
    pub(crate) fn bump_revision(&mut self) {
        self.revision += 1;
    }

    pub(crate) fn species(&self) -> &Arc<SpeciesDefinition> {
        &self.species
    }

    pub(crate) fn world(&self) -> &Arc<WorldDefinition> {
        &self.world
    }

    pub(crate) fn appearance(&self) -> &Appearance {
        &self.appearance
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for id in [self.id(), self.species.id()]
            .into_iter()
            .chain(self.appearance.materials().values().map(|m| m.id()))
        {
            if id.get_version() != Some(Version::SortRand) || !ids.insert(id) {
                return Err("Les identités du catalogue doivent être des UUIDv7 uniques.".into());
            }
        }

        let w = &self.world;
        if !(1..=128).contains(&w.size()) {
            return Err(
                "La taille du monde doit être comprise entre 1 et 128 habitats par axe.".into(),
            );
        }

        let r = &self.species.rules();
        if [
            r.energy_capacity(),
            r.basal_metabolism(),
            r.movement_seconds(),
            r.movement_energy_seconds(),
            w.food_capacity(),
            w.food_regeneration_per_second(),
            self.appearance.cell_radius(),
            self.appearance.habitat_spacing(),
        ]
        .iter()
        .any(|n| !n.is_finite() || *n <= 0.0)
            || r.maturity_seconds() == 0
            || !r.movement_reserve().is_finite()
            || r.movement_reserve() < 0.0
            || r.movement_reserve() >= r.energy_capacity()
            || 2.0 * r.basal_metabolism() * r.maturity_seconds() as f32 > r.energy_capacity()
        {
            return Err("Budgets énergétiques, cadence et géométrie incohérents.".into());
        }

        self.species.validate_genetics()?;

        let mut positions = HashSet::new();
        for placement in w.placements() {
            if placement.x() >= w.size()
                || placement.y() >= w.size()
                || !positions.insert((placement.x(), placement.y()))
            {
                return Err("Placement hors du monde ou dupliqué.".into());
            }

            let valid = match placement.entity() {
                crate::habitat::Entity::Food(n) => {
                    n.is_finite() && (0.0..=w.food_capacity()).contains(n)
                }
                crate::habitat::Entity::HeatSource(n) => n.is_finite() && *n >= 0.0,
                crate::habitat::Entity::Water(n) | crate::habitat::Entity::Oxygen(n) => {
                    n.is_finite() && (0.0..=1.0).contains(n)
                }
                _ => true,
            };

            if !valid {
                return Err("Ressource définie invalide.".into());
            }
        }

        for p in [
            w.terrain_probability(),
            w.oxygen_probability(),
            w.water_probability(),
            w.resource_spread_probability(),
        ] {
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                return Err("Probabilité hors de [0, 1].".into());
            }
        }

        if !w.ambient_temperature().is_finite()
            || !w.water_decay_per_second().is_finite()
            || !(0.0..=1.0).contains(&w.water_decay_per_second())
            || !w.heat_intensity()[0].is_finite()
            || !w.heat_intensity()[1].is_finite()
            || w.heat_intensity()[0] < 0.0
            || w.heat_intensity()[0] >= w.heat_intensity()[1]
            || w.food_attempts() > 100_000
            || w.heat_sources() > 100_000
        {
            return Err("Paramètres du monde incohérents.".into());
        }

        for kind in [
            MaterialKind::Plinth,
            MaterialKind::Rim,
            MaterialKind::Ground,
            MaterialKind::Soil,
            MaterialKind::Sand,
            MaterialKind::Rock,
            MaterialKind::Water,
            MaterialKind::Food,
            MaterialKind::Heat,
            MaterialKind::Oxygen,
            MaterialKind::Iron,
            MaterialKind::Calcium,
            MaterialKind::Phosphorus,
            MaterialKind::Healthy,
            MaterialKind::Starving,
            MaterialKind::Nucleus,
            MaterialKind::Selection,
        ] {
            let material = self
                .appearance()
                .materials()
                .get(&kind)
                .ok_or_else(|| format!("Matériau absent : {kind:?}"))?;

            if !material.roughness().is_finite()
                || !(0.0..=1.0).contains(&material.roughness())
                || !material.metallic().is_finite()
                || !(0.0..=1.0).contains(&material.metallic())
                || !material.emission().is_finite()
                || material.emission() < 0.0
            {
                return Err(format!("Matériau invalide : {kind:?}"));
            }
        }

        Ok(())
    }

    /// Empreinte de la partie biologique du catalogue : identité, règles de l'espèce, génétique et monde.
    /// L'apparence (titre, matériaux, géométrie de rendu) n'y figure pas : modifier la
    /// présentation ne doit pas invalider une sauvegarde de l'expérience. Le génome de l'ancêtre n'y
    /// figure pas non plus : il ne sert qu'à créer l'ancêtre d'une nouvelle expérience, jamais à
    /// restaurer des organismes qui portent leur propre génome.
    pub(crate) fn fingerprint(&self) -> Result<u64, String> {
        let species = &self.species;
        Self::hash(&(
            self.id,
            species.id(),
            species.name(),
            species.genetics_definition(),
            species.rules(),
            &self.world,
        ))
    }

    fn hash(value: &impl Serialize) -> Result<u64, String> {
        // FNV-1a du catalogue canonique, pour détecter les modifications, pas pour authentifier un fichier.
        let bytes = bincode_next::serde::encode_to_vec(value, bincode_next::config::standard())
            .map_err(|e| e.to_string())?;

        Ok(bytes.into_iter().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ byte as u64).wrapping_mul(0x100000001b3)
        }))
    }
}
