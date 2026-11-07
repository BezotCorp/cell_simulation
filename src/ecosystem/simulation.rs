use crate::{
    biology::Cell,
    definitions::Definitions,
    ecosystem::{
        FIXED_STEP, FoodCompetition, Lineage, LineageRecord, PopulationSummary, SimulationRng,
    },
    habitat::Grid,
    hydrology::Hydrology,
};
use rand::seq::SliceRandom;
use std::{collections::HashSet, sync::Arc, time::Duration};
use uuid::{Uuid, Version};

fn default_definitions() -> Arc<Definitions> {
    Definitions::builtin()
}

const ONE_SECOND: Duration = Duration::from_secs(1);

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Simulation {
    grid: Grid,
    cells: Vec<Cell>,
    generation: u32,
    rng: SimulationRng,
    id: Uuid,
    seed: u64,
    ticks: u64,
    lineage: Lineage,
    #[serde(skip, default = "default_definitions")]
    definitions: Arc<Definitions>,
    /// Tampons de travail réutilisés à chaque pas ; ils ne font pas partie de l'état biologique.
    #[serde(skip)]
    competition: FoodCompetition,
    #[serde(skip)]
    maintenance: Vec<f32>,
}

impl Simulation {
    pub(crate) fn from_definitions(
        definitions: Arc<Definitions>,
        seed: u64,
    ) -> Result<Self, String> {
        definitions.validate()?;

        let mut rng = SimulationRng::new(seed);
        let grid = Grid::new(definitions.world().clone(), &mut rng);

        let cells: Vec<Cell> = grid
            .initial_position()
            .map(|(x, y)| vec![Cell::new(x, y, definitions.species().clone())])
            .unwrap_or_default();

        let mut lineage = Lineage::default();
        for cell in &cells {
            lineage.record_birth(cell.id(), None, 0);
        }

        Ok(Self {
            grid,
            cells,
            generation: 0,
            rng,
            id: Uuid::now_v7(),
            seed,
            ticks: 0,
            lineage,
            definitions,
            competition: FoodCompetition::default(),
            maintenance: Vec::new(),
        })
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    pub(crate) fn ticks(&self) -> u64 {
        self.ticks
    }

    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }

    pub(crate) fn definitions(&self) -> &Arc<Definitions> {
        &self.definitions
    }

    pub(crate) fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Organisme vivant d'identité donnée (recherche linéaire : réservée à l'inspection).
    pub(crate) fn cell(&self, id: Uuid) -> Option<&Cell> {
        self.cells.iter().find(|cell| cell.id() == id)
    }

    /// Vivacité en temps constant, d'après la filiation.
    pub(crate) fn is_alive(&self, id: Uuid) -> bool {
        self.lineage.is_alive(id)
    }

    pub(crate) fn grid(&self) -> &Grid {
        &self.grid
    }

    pub(crate) fn lineage(&self) -> &[LineageRecord] {
        self.lineage.records()
    }

    pub(crate) fn lineage_record(&self, id: Uuid) -> Option<&LineageRecord> {
        self.lineage.get(id)
    }

    pub(crate) fn summary(&self) -> PopulationSummary {
        PopulationSummary::new(
            self.cells.len(),
            self.generation as u64,
            self.lineage.deaths(),
            self.ticks as f64 * FIXED_STEP.as_secs_f64(),
            if self.cells.is_empty() {
                0.0
            } else {
                self.cells.iter().map(|cell| cell.energy()).sum::<f32>() / self.cells.len() as f32
            },
            self.grid.water_level(),
        )
    }

    /// Contrôle sans rien modifier que l'état restauré est cohérent avec `definitions`.
    /// Renvoie l'index de filiation, que [`Simulation::bind_definitions`] installe ensuite.
    pub(crate) fn verify(
        &self,
        definitions: &Definitions,
    ) -> Result<std::collections::HashMap<Uuid, usize>, String> {
        definitions.validate()?;
        self.grid.verify_definition(definitions.world())?;
        if self.grid.has_hydrology() {
            self.grid.hydrology().verify(self.grid.size())?;
        }

        let mut ids = HashSet::new();

        for id in [self.id].into_iter().chain(self.grid.habitat_ids()) {
            if id.get_version() != Some(Version::SortRand) || !ids.insert(id) {
                return Err("Identité persistante invalide ou dupliquée.".into());
            }
        }

        let index = self
            .lineage
            .verify(self.ticks, self.generation as u64, &ids)?;

        let mut alive = HashSet::new();

        for cell in &self.cells {
            let record = index
                .get(&cell.id())
                .map(|&position| &self.lineage.records()[position])
                .ok_or("Organisme absent de la filiation")?;

            if !alive.insert(cell.id())
                || record.died_at_tick().is_some()
                || cell.x() >= self.grid.size()
                || cell.y() >= self.grid.size()
                || !self.grid.is_passable(cell.x(), cell.y())
                || cell.age().as_nanos()
                    != (self.ticks - record.born_at_tick()) as u128 * FIXED_STEP.as_nanos()
            {
                return Err("Organisme invalide ou situé dans un obstacle.".into());
            }

            let (east, south) = cell.offsets();
            if !Cell::offsets_are_valid(east, south) {
                return Err("Position fine d'un organisme invalide.".into());
            }

            cell.verify_species(definitions.species())?;
        }

        if self
            .lineage
            .records()
            .iter()
            .any(|record| record.died_at_tick().is_none() && !alive.contains(&record.id()))
        {
            return Err("Organisme vivant absent du monde.".into());
        }

        Ok(index)
    }

    /// Positions fines des organismes (identité, est, sud), dans l'ordre du monde : section
    /// dédiée du fichier BIN.
    pub(crate) fn positions(&self) -> Vec<(Uuid, f32, f32)> {
        self.cells
            .iter()
            .map(|cell| {
                let (east, south) = cell.offsets();
                (cell.id(), east, south)
            })
            .collect()
    }

    /// Restaure les positions fines lues dans une sauvegarde ; une erreur n'en applique aucune.
    pub(crate) fn restore_positions(&mut self, positions: &[(Uuid, f32, f32)]) -> Result<(), String> {
        if positions.len() != self.cells.len()
            || self.cells.iter().zip(positions).any(|(cell, (id, east, south))| {
                cell.id() != *id || !Cell::offsets_are_valid(*east, *south)
            })
        {
            return Err("Positions des organismes incohérentes avec le monde.".into());
        }
        for (cell, (_, east, south)) in self.cells.iter_mut().zip(positions) {
            cell.restore_offsets(*east, *south);
        }
        Ok(())
    }

    /// Anciennes sauvegardes : position fine dérivée de l'identité de chaque organisme.
    pub(crate) fn settle_legacy_positions(&mut self) {
        for cell in &mut self.cells {
            cell.settle_legacy_offsets();
        }
    }

    /// Installe l'état hydrologique lu dans une sauvegarde.
    pub(crate) fn restore_hydrology(&mut self, hydrology: Hydrology) -> Result<(), String> {
        self.grid.install_hydrology(hydrology)
    }

    /// Anciennes sauvegardes : l'hydrologie est reconstruite depuis le niveau d'eau enregistré.
    pub(crate) fn settle_legacy_hydrology(&mut self) {
        self.grid.rebuild_legacy_hydrology(self.seed ^ 0x5DEE_CE66_D1CE_4E5B);
    }

    /// Valide puis adopte `definitions` : une erreur laisse la simulation strictement intacte.
    pub(crate) fn bind_definitions(&mut self, definitions: Arc<Definitions>) -> Result<(), String> {
        let index = self.verify(&definitions)?;

        // Tout est vérifié : les étapes suivantes ne peuvent plus échouer.
        self.lineage.adopt_index(index);
        self.grid.adopt_definition(definitions.world().clone());
        for cell in &mut self.cells {
            cell.adopt_species(definitions.species().clone());
        }
        self.definitions = definitions;

        Ok(())
    }

    pub(crate) fn step(&mut self, dt: Duration) {
        assert_eq!(
            dt, FIXED_STEP,
            "le domaine avance exclusivement au pas fixe de 16 ms"
        );

        self.begin_step(dt);
        self.act(dt);
        self.feed_and_divide();
        self.bury_dead();
    }

    /// Évolue l'environnement, mélange l'ordre de passage et mesure la charge d'entretien par site.
    fn begin_step(&mut self, dt: Duration) {
        self.ticks += 1;
        self.grid.update(dt, &mut self.rng);
        self.cells.shuffle(&mut self.rng);

        // Toutes les intentions lisent les mêmes réserves avant ingestion.
        let size = self.grid.size();
        self.competition.reset(size);
        self.maintenance.clear();
        self.maintenance.resize(size * size, 0.0);

        for cell in &self.cells {
            self.maintenance[cell.y() * size + cell.x()] += cell.maintenance_cost(
                self.grid.get_environment_type(cell.x(), cell.y()),
                ONE_SECOND,
            );
        }
    }

    /// Métabolisme, décision de déplacement et demandes de nourriture de chaque organisme.
    fn act(&mut self, dt: Duration) {
        let size = self.grid.size();

        for cell in &mut self.cells {
            // Le courant s'exerce avant toute décision : l'organisme est d'abord emporté ou retenu.
            if let Some((east, south)) =
                cell.swim_against(self.grid.flow_at(cell.x(), cell.y()), dt)
            {
                let from = cell.position();
                let to = (from.0 + east, from.1 + south);
                if self.grid.can_travel(from, to) {
                    let old = cell.y() * size + cell.x();
                    let old_rate = cell.maintenance_cost(
                        self.grid.get_environment_type(cell.x(), cell.y()),
                        ONE_SECOND,
                    );
                    cell.settle(to, size);
                    let new = cell.y() * size + cell.x();
                    if new != old {
                        self.maintenance[old] = (self.maintenance[old] - old_rate).max(0.0);
                        self.maintenance[new] += cell.maintenance_cost(
                            self.grid.get_environment_type(cell.x(), cell.y()),
                            ONE_SECOND,
                        );
                    }
                }
            }
            let food = self.grid.food_energy(cell.x(), cell.y());

            let wants_to_move = cell.update(
                self.grid.temperature_at(cell.position()),
                self.grid.get_environment_type(cell.x(), cell.y()),
                food,
                dt,
            );

            if cell.energy() <= 0.0 {
                continue;
            }

            if wants_to_move {
                let old_index = cell.y() * size + cell.x();

                let rate = cell.maintenance_cost(
                    self.grid.get_environment_type(cell.x(), cell.y()),
                    ONE_SECOND,
                );

                if let Some(destination) = self.grid.exploration_destination(
                    cell.position(),
                    &self.maintenance,
                    rate,
                    &mut self.rng,
                ) {
                    let (nx, ny) = self.grid.site_at(destination);
                    // Réserver le site pour que les décisions suivantes voient les arrivées/départs.
                    self.maintenance[old_index] = (self.maintenance[old_index] - rate).max(0.0);

                    cell.move_to(destination, size);

                    self.maintenance[ny * size + nx] +=
                        cell.maintenance_cost(self.grid.get_environment_type(nx, ny), ONE_SECOND);
                }
            }

            if cell.energy() <= 0.0 {
                continue;
            }

            self.competition
                .request(cell.x(), cell.y(), cell.energy_deficit());
        }
    }

    /// Répartit la nourriture puis fait diviser les organismes éligibles.
    fn feed_and_divide(&mut self) {
        self.competition.consume_from(&mut self.grid);

        let size = self.grid.size();
        let mut daughters = Vec::new();

        for cell in &mut self.cells {
            if cell.energy() <= 0.0 {
                continue;
            }

            cell.feed(
                self.competition
                    .share(cell.x(), cell.y(), cell.energy_deficit()),
            );

            if let Some(mut daughter) = cell.try_divide(&mut self.rng) {
                // La fille naît à portée de sa mère : le voisinage est colonisé de proche en proche.
                let birth = self.grid.birth_point(cell.position(), &mut self.rng);
                daughter.settle(birth, size);
                self.lineage
                    .record_birth(daughter.id(), Some(cell.id()), self.ticks);
                daughters.push(daughter);
                self.generation += 1;
            }
        }

        self.cells.extend(daughters);
    }

    /// Retire les organismes sans énergie en conservant leur filiation.
    fn bury_dead(&mut self) {
        let ticks = self.ticks;
        let lineage = &mut self.lineage;

        self.cells.retain(|cell| {
            let alive = cell.energy() > 0.0;
            if !alive {
                lineage.record_death(cell.id(), ticks);
            }
            alive
        });
    }
}
