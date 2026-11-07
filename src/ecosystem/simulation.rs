use crate::{
    biology::{Cell, Conditions},
    genetics::Product,
    definitions::Definitions,
    ecosystem::{
        Bonds, FIXED_STEP, Hunting, FoodCompetition, HarvestShare, Lineage, LineageRecord,
        PopulationSummary, SimulationRng,
    },
    habitat::{Environment, Grid},
    hydrology::{Hydrology, Storm, Strike},
};
use rand::{RngExt, seq::SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use uuid::{Uuid, Version};

fn default_definitions() -> Arc<Definitions> {
    Definitions::builtin()
}

/// Sensibilité du taux de mutation au rayonnement ultraviolet (lumière non filtrée) et à la
/// chaleur : leur exposition l'augmente, la réparation du génome le diminue.
const UV_MUTAGENESIS: f32 = 0.6;
const HEAT_MUTAGENESIS: f32 = 0.8;
const ONE_SECOND: Duration = Duration::from_secs(1);
/// Organismes à photosynthèse pleine que la lumière d'un site peut nourrir ; au-delà ils s'ombragent.
const LIGHT_CAPACITY: f32 = 3.0;
/// Idem pour la chaleur d'un site (thermosynthèse).
const HEAT_CAPACITY: f32 = 2.0;
/// Nombre maximal de liaisons par organisme.
const MAX_BONDS: u8 = 4;
/// Distance de repos d'une liaison, distance de rupture, raideur et débit de partage d'énergie.
const BOND_REST: f32 = 0.45;
const BOND_BREAK: f32 = 1.8;
const BOND_STIFFNESS: f32 = 0.1;
const BOND_SHARING: f32 = 0.2;
/// Énergie retirée par unité de charge d'un éclair à son point d'impact, et portée de l'éclair.
const STRIKE_DAMAGE: f32 = 150.0;
const STRIKE_REACH: f32 = 1.5;
/// Éclairs conservés pour l'affichage.
const RECENT_STRIKES: usize = 16;

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
    /// Nombre d'organismes par site au début du pas (densité perçue par les gènes).
    #[serde(skip)]
    occupancy: Vec<u16>,
    /// Liaisons entre organismes : section dédiée du fichier BIN (version 6).
    #[serde(skip)]
    bonds: Bonds,
    /// Lumière et chaleur partagées entre les occupants de chaque site.
    #[serde(skip, default = "light_share")]
    light: HarvestShare,
    #[serde(skip, default = "heat_share")]
    heat: HarvestShare,
    /// Derniers éclairs (rendu et inspection) : dérivés, non sauvegardés.
    #[serde(skip)]
    strikes: Vec<Strike>,
    /// Tampons de la chasse, réutilisés d'un pas à l'autre.
    #[serde(skip)]
    hunting: Hunting,
}

fn light_share() -> HarvestShare {
    HarvestShare::new(LIGHT_CAPACITY)
}

fn heat_share() -> HarvestShare {
    HarvestShare::new(HEAT_CAPACITY)
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
            occupancy: Vec::new(),
            bonds: Bonds::default(),
            light: light_share(),
            heat: heat_share(),
            strikes: Vec::new(),
            hunting: Hunting::default(),
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


    /// Un éclair décharge son énergie dans l'eau et le sol alentour : les organismes proches perdent
    /// de l'énergie, d'autant moins qu'ils sont loin, et deux fois plus s'ils baignent dans l'eau
    /// qui conduit la décharge. Ceux qu'elle vide meurent comme de faim, avec leur filiation.
    fn apply_strikes(&mut self, strikes: &[Strike]) {
        if strikes.is_empty() {
            return;
        }
        let size = self.grid.size() as f32;
        for strike in strikes {
            let center = (strike.x() as f32 + 0.5, strike.y() as f32 + 0.5);
            for cell in &mut self.cells {
                let position = cell.position();
                let wrap = |a: f32, b: f32| {
                    let d = (a - b).abs();
                    d.min(size - d)
                };
                let distance = wrap(position.0, center.0).hypot(wrap(position.1, center.1));
                if distance >= STRIKE_REACH {
                    continue;
                }
                let conduction = match self.grid.get_environment_type(cell.x(), cell.y()) {
                    Environment::Water => 2.0,
                    Environment::Land => 1.0,
                };
                cell.damage(strike.energy() * STRIKE_DAMAGE * conduction * (1.0 - distance / STRIKE_REACH));
            }
        }
        self.strikes.extend_from_slice(strikes);
        let excess = self.strikes.len().saturating_sub(RECENT_STRIKES);
        self.strikes.drain(..excess);
    }

    /// Prédation : un organisme qui l'exprime prélève de l'énergie sur un voisin du même site, hors
    /// organismes liés à lui. Le prélèvement est exactement ce que perd la proie ; le prédateur n'en
    /// garde que le rendement. Rien ne se calcule tant qu'aucun organisme ne chasse.
    fn hunt(&mut self, dt: Duration) {
        if !self.cells.iter().any(|cell| cell.predation() > 0.0) {
            return;
        }
        let size = self.grid.size();
        // Tri par site en temps linéaire : `heads[s]..heads[s + 1]` délimite les occupants du site `s`.
        let site = |cell: &Cell| cell.y() * size + cell.x();
        self.hunting.heads.clear();
        self.hunting.heads.resize(size * size + 1, 0);
        for cell in &self.cells {
            self.hunting.heads[site(cell) + 1] += 1;
        }
        for index in 0..size * size {
            self.hunting.heads[index + 1] += self.hunting.heads[index];
        }
        self.hunting.members.clear();
        self.hunting.members.resize(self.cells.len(), 0);
        self.hunting.fill.clear();
        self.hunting.fill.extend_from_slice(&self.hunting.heads[..size * size]);
        for (index, cell) in self.cells.iter().enumerate() {
            let slot = &mut self.hunting.fill[site(cell)];
            self.hunting.members[*slot as usize] = index as u32;
            *slot += 1;
        }

        for predator in 0..self.cells.len() {
            let hunter = &self.cells[predator];
            if hunter.predation() <= 0.0 || hunter.energy_deficit() <= 0.0 || hunter.energy() <= 0.0 {
                continue;
            }
            let here = site(hunter);
            let range = self.hunting.heads[here] as usize..self.hunting.heads[here + 1] as usize;
            let id = hunter.id();
            let eligible = |other: u32| {
                let other = other as usize;
                let prey = &self.cells[other];
                other != predator && prey.energy() > 0.0 && !self.bonds.are_linked(id, prey.id())
            };
            let count = self.hunting.members[range.clone()].iter().filter(|&&o| eligible(o)).count();
            if count == 0 {
                continue;
            }
            let pick = self.rng.random_range(0..count);
            let prey = self.hunting.members[range]
                .iter()
                .copied()
                .filter(|&o| eligible(o))
                .nth(pick)
                .expect("le décompte garantit l'existence de la proie") as usize;
            let wanted = self.cells[predator].bite(dt);
            let taken = self.cells[prey].be_bitten(wanted);
            self.cells[predator].digest(taken);
        }
    }

    /// Liaisons : les organismes liés partagent leur énergie (de l'un à l'autre, sans perte) et se
    /// rapprochent quand ils s'écartent ; une liaison se rompt si la distance dépasse sa portée, si
    /// l'un des deux n'exprime plus l'adhésion ou s'il est mort.
    fn bind(&mut self, dt: Duration) {
        if self.bonds.is_empty() {
            return;
        }
        let size = self.grid.size();
        let extent = size as f32;
        let seconds = dt.as_secs_f32();
        let index: HashMap<Uuid, usize> = self
            .cells
            .iter()
            .enumerate()
            .map(|(position, cell)| (cell.id(), position))
            .collect();
        let (cells, grid) = (&mut self.cells, &self.grid);
        self.bonds.retain(|a, b| {
            let (Some(&ia), Some(&ib)) = (index.get(&a), index.get(&b)) else {
                return false;
            };
            if ia == ib || cells[ia].energy() <= 0.0 || cells[ib].energy() <= 0.0 {
                return false;
            }
            let strength = (cells[ia].adhesion() * cells[ib].adhesion()).sqrt();
            let (pa, pb) = (cells[ia].position(), cells[ib].position());
            let wrap = |d: f32| (d + extent / 2.0).rem_euclid(extent) - extent / 2.0;
            let (dx, dy) = (wrap(pb.0 - pa.0), wrap(pb.1 - pa.1));
            let distance = dx.hypot(dy);
            if strength <= 0.0 || distance > BOND_BREAK {
                return false;
            }
            // Partage : l'énergie va de la plus riche à la plus pauvre, dans la limite de ce que
            // l'une peut donner et de ce que l'autre peut recevoir.
            let wanted = BOND_SHARING * seconds * (cells[ia].energy() - cells[ib].energy()) * strength;
            let flow = wanted.clamp(
                -cells[ib].energy().min(cells[ia].energy_deficit()),
                cells[ia].energy().min(cells[ib].energy_deficit()),
            );
            cells[ia].exchange(-flow);
            cells[ib].exchange(flow);
            // Rappel élastique vers la distance de repos, sans traverser de solide.
            if distance > BOND_REST {
                let pull = (distance - BOND_REST) * BOND_STIFFNESS / distance;
                let toward_b = (pa.0 + dx * pull, pa.1 + dy * pull);
                let toward_a = (pb.0 - dx * pull, pb.1 - dy * pull);
                if grid.can_travel(pa, toward_b) {
                    cells[ia].settle(toward_b, size);
                }
                if grid.can_travel(pb, toward_a) {
                    cells[ib].settle(toward_a, size);
                }
            }
            true
        });
    }

    /// Derniers éclairs, du plus ancien au plus récent.
    pub(crate) fn strikes(&self) -> &[Strike] {
        &self.strikes
    }

    pub(crate) fn bonds(&self) -> &Bonds {
        &self.bonds
    }

    /// Nombre d'organismes qui expriment chaque produit à au moins `threshold`.
    pub(crate) fn census(&self, threshold: f32) -> [usize; Product::COUNT] {
        let mut counts = [0; Product::COUNT];
        for cell in &self.cells {
            for product in Product::ALL {
                counts[product.index()] += usize::from(cell.levels().get(product) >= threshold);
            }
        }
        counts
    }

    /// Restaure les liaisons lues dans une sauvegarde ; une erreur n'en applique aucune.
    pub(crate) fn restore_bonds(&mut self, bonds: Vec<(Uuid, Uuid)>) -> Result<(), String> {
        let living: HashSet<Uuid> = self.cells.iter().map(Cell::id).collect();
        if bonds.iter().any(|(a, b)| a == b || !living.contains(a) || !living.contains(b)) {
            return Err("Liaison vers un organisme absent.".into());
        }
        self.bonds = Bonds::from_pairs(bonds);
        Ok(())
    }

    /// Liaisons enregistrées, dans leur ordre.
    pub(crate) fn bond_pairs(&self) -> Vec<(Uuid, Uuid)> {
        self.bonds.pairs().to_vec()
    }

    /// Installe l'état orageux lu dans une sauvegarde.
    pub(crate) fn restore_storm(&mut self, storm: Storm) -> Result<(), String> {
        self.grid.install_storm(storm)
    }

    /// Reconstitue une simulation à partir des éléments d'un ancien fichier ; les organismes ont déjà
    /// été convertis par l'appelant.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        grid: Grid,
        cells: Vec<Cell>,
        generation: u32,
        rng: SimulationRng,
        id: Uuid,
        seed: u64,
        ticks: u64,
        lineage: Lineage,
        definitions: Arc<Definitions>,
    ) -> Self {
        Self {
            grid,
            cells,
            generation,
            rng,
            id,
            seed,
            ticks,
            lineage,
            definitions,
            competition: FoodCompetition::default(),
            maintenance: Vec::new(),
            occupancy: Vec::new(),
            bonds: Bonds::default(),
            light: light_share(),
            heat: heat_share(),
            strikes: Vec::new(),
            hunting: Hunting::default(),
        }
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
        self.hunt(dt);
        self.bind(dt);
        self.feed_and_divide();
        self.bury_dead();
    }

    /// Évolue l'environnement, mélange l'ordre de passage et mesure la charge d'entretien par site.
    fn begin_step(&mut self, dt: Duration) {
        self.ticks += 1;
        let strikes = self.grid.update(dt, &mut self.rng);
        self.cells.shuffle(&mut self.rng);
        self.apply_strikes(&strikes);

        // Toutes les intentions lisent les mêmes réserves avant ingestion.
        let size = self.grid.size();
        self.competition.reset(size);
        self.maintenance.clear();
        self.maintenance.resize(size * size, 0.0);
        self.occupancy.clear();
        self.occupancy.resize(size * size, 0);
        self.light.reset(size);
        self.heat.reset(size);

        for cell in &self.cells {
            if cell.photosynthesis() > 0.0 {
                self.light.request(cell.x(), cell.y(), cell.photosynthesis());
            }
            if cell.thermosynthesis() > 0.0 {
                self.heat.request(cell.x(), cell.y(), cell.thermosynthesis());
            }
            self.occupancy[cell.y() * size + cell.x()] += 1;
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
            let site = cell.y() * size + cell.x();
            // Température et lumière ne se calculent que si quelqu'un s'en sert : les gènes les
            // relisent une fois par seconde de vie, la captation et le capteur à chaque pas.
            let refreshing = cell.will_refresh(dt);
            let needs_light = refreshing || cell.photosynthesis() > 0.0 || cell.senses_light();
            let needs_heat = refreshing || cell.thermosynthesis() > 0.0 || cell.senses_temperature();
            let temperature = if needs_heat {
                self.grid.temperature_at(cell.position())
            } else {
                0.0
            };
            let ambient = self.grid.temperature_map().ambient_temperature();
            let light = if needs_light {
                self.grid.light_at(cell.x(), cell.y()) * self.light.scale(cell.x(), cell.y())
            } else {
                0.0
            };
            let conditions = Conditions::new(
                temperature,
                (temperature - ambient).max(0.0) * self.heat.scale(cell.x(), cell.y()),
                light,
                self.grid.get_environment_type(cell.x(), cell.y()),
                self.grid.food_energy(cell.x(), cell.y()),
                self.grid.food_capacity(),
                f32::from(self.occupancy[site]) - 1.0,
            );

            let wants_to_move = cell.update(&conditions, dt);

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

            self.competition.request(
                cell.x(),
                cell.y(),
                cell.energy_deficit() * cell.ingestion(),
            );
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

            cell.feed(self.competition.share(
                cell.x(),
                cell.y(),
                cell.energy_deficit() * cell.ingestion(),
            ));

            // Rayonnement ultraviolet et chaleur abîment l'ADN : le taux de mutation de la division
            // dépend du milieu de la mère, jamais de ce qui serait utile à la fille.
            let mother = site_conditions(&self.grid, &self.occupancy, cell.position());
            let exposure = 1.0 + UV_MUTAGENESIS * mother.light() + HEAT_MUTAGENESIS * mother.warmth() / 30.0;
            if let Some(mut daughter) = cell.try_divide(&mut self.rng, exposure) {
                // La fille naît à portée de sa mère : le voisinage est colonisé de proche en proche.
                let birth = self.grid.birth_point(cell.position(), &mut self.rng);
                daughter.settle(birth, size);
                // Son corps se développe selon le milieu où elle naît.
                let born_in = site_conditions(&self.grid, &self.occupancy, daughter.position());
                daughter.develop(born_in.signals(daughter.energy() / daughter.capacity(), 0.0));
                // Une cellule adhésive reste liée à sa fille, avec une probabilité égale à son
                // adhésion : le tirage n'a lieu que pour ces cellules, le rejeu des autres est intact.
                if cell.adhesion() > 0.0
                    && self.bonds.degree(cell.id()) < MAX_BONDS
                    && self.rng.random::<f32>() < cell.adhesion()
                {
                    self.bonds.link(cell.id(), daughter.id());
                }
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

/// Conditions du site contenant `point`, sans partage de la lumière ni de la chaleur entre
/// occupants : ce que perçoit un organisme qui s'y trouve seul.
fn site_conditions(grid: &Grid, occupancy: &[u16], point: (f32, f32)) -> Conditions {
    let (x, y) = grid.site_at(point);
    let temperature = grid.temperature_at(point);
    let ambient = grid.temperature_map().ambient_temperature();
    Conditions::new(
        temperature,
        (temperature - ambient).max(0.0),
        grid.light_at(x, y),
        grid.get_environment_type(x, y),
        grid.food_energy(x, y),
        grid.food_capacity(),
        f32::from(occupancy[y * grid.size() + x]),
    )
}
