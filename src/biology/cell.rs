use crate::definitions::{Definitions, SpeciesDefinition};
use std::sync::Arc;
use uuid::Uuid;

fn default_species() -> Arc<SpeciesDefinition> {
    Definitions::builtin().species().clone()
}

fn centered() -> f32 {
    0.5
}
use std::time::Duration;

use crate::biology::{Conditions, Morphology, Physiology};
use crate::genetics::{Genome, Levels, Signals};
use crate::habitat::Environment;
use crate::hydrology::Flow;

/// Part du métabolisme dépensée en plus pour tenir tête, à pleine poussée, au courant.
const SWIM_COST: f32 = 0.3;
/// Énergie captée par seconde par un organisme de photosynthèse 1 recevant toute la lumière.
const PHOTOSYNTHESIS_YIELD: f32 = 3.6;
/// Énergie captée par seconde et par tranche de 10 °C au-dessus de l'ambiante, à thermosynthèse 1.
const THERMOSYNTHESIS_YIELD: f32 = 1.2;
/// Excès de température au-delà duquel la thermosynthèse ne rapporte plus davantage.
const THERMOSYNTHESIS_SATURATION: f32 = 30.0;
/// Énergie prélevée par seconde sur une proie par un organisme de prédation 1.
const PREDATION_RATE: f32 = 4.0;
/// Part de l'énergie prélevée qui profite au prédateur (le reste est dissipé).
const PREDATION_EFFICIENCY: f32 = 0.45;
/// En dessous de cette motilité l'organisme est sessile.
const SESSILE_MOTILITY: f32 = 0.05;

/// Un organisme : son génotype héritable, ce qui en a été exprimé, et son état physiologique. Sa
/// forme (`Morphology`) et son fonctionnement (`Physiology`) se déduisent du génome, des conditions
/// de sa naissance et du milieu qu'il traverse ; ils ne sont jamais sauvegardés eux-mêmes.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Cell {
    id: Uuid,
    species_id: Uuid,
    #[serde(skip, default = "default_species")]
    species: Arc<SpeciesDefinition>,
    /// Site de la grille technique qui contient l'organisme (index de calcul, pas un enclos).
    x: usize,
    y: usize,
    /// Position réelle dans le site, en fraction de sa largeur. Elle est écrite dans une section
    /// dédiée du fichier BIN afin que le corps de la session garde son format historique.
    #[serde(skip, default = "centered")]
    east: f32,
    #[serde(skip, default = "centered")]
    south: f32,
    energy: f32,
    genome: Genome,
    /// Signaux du milieu de naissance : ils règlent le développement, donc la morphologie.
    birth: Signals,
    /// Produits exprimés dans le milieu courant, rafraîchis une fois par seconde de vie.
    levels: Levels,
    age: Duration,
    last_division: Duration,
    movement_elapsed: Duration,
    #[serde(skip)]
    morphology: Option<Morphology>,
    #[serde(skip)]
    physiology: Option<Physiology>,
    #[serde(skip)]
    activation: f32,
}

impl Cell {
    /// Fondateur : le génome du catalogue, exprimé sans signal particulier.
    pub(crate) fn new(x: usize, y: usize, species: Arc<SpeciesDefinition>) -> Self {
        Self::born(x, y, species.genome().to_genome(), Signals::default(), species)
    }

    /// Organisme né avec `genome` dans un milieu de signaux `birth`.
    fn born(
        x: usize,
        y: usize,
        genome: Genome,
        birth: Signals,
        species: Arc<SpeciesDefinition>,
    ) -> Self {
        let levels = genome.express(&birth);
        let mut cell = Self {
            id: Uuid::now_v7(),
            species_id: species.id(),
            species,
            x,
            y,
            east: 0.5,
            south: 0.5,
            energy: 0.0,
            genome,
            birth,
            levels,
            age: Duration::from_secs(0),
            last_division: Duration::from_secs(0),
            movement_elapsed: Duration::ZERO,
            morphology: None,
            physiology: None,
            activation: 0.0,
        };
        cell.develop_body();
        cell.energy = cell.morphology().capacity();
        cell
    }

    /// Construit (ou reconstruit) forme et fonctionnement d'après le génome et l'état exprimé.
    fn develop_body(&mut self) {
        let morphology = Morphology::develop(
            self.genome.express(&self.birth),
            self.species.rules().energy_capacity(),
        );
        self.physiology = Some(Physiology::express(&self.levels, &morphology, self.genome.genes().len()));
        self.morphology = Some(morphology);
    }

    pub(crate) fn morphology(&self) -> &Morphology {
        self.morphology.as_ref().expect("la forme est construite avec l'organisme")
    }

    pub(crate) fn physiology(&self) -> &Physiology {
        self.physiology.as_ref().expect("le fonctionnement est construit avec l'organisme")
    }

    pub(crate) fn genome(&self) -> &Genome {
        &self.genome
    }

    /// Produits exprimés dans le milieu courant.
    pub(crate) fn levels(&self) -> &Levels {
        &self.levels
    }

    pub(crate) fn birth(&self) -> &Signals {
        &self.birth
    }

    /// Réserve d'énergie maximale de cet organisme.
    pub(crate) fn capacity(&self) -> f32 {
        self.morphology().capacity()
    }

    /// Réserve à atteindre pour se diviser, jamais en deçà de ce qu'exigent deux maturations.
    pub(crate) fn division_threshold(&self) -> f32 {
        let floor = 2.0
            * self.species.rules().basal_metabolism()
            * self.species.rules().maturity_seconds() as f32;
        (self.capacity() * self.physiology().division_fraction()).clamp(floor.min(self.capacity()), self.capacity())
    }

    /// Dernière activation du réseau neuronal.
    pub(crate) fn activation(&self) -> f32 {
        self.activation
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn species_id(&self) -> Uuid {
        self.species_id
    }
    /// Contrôle sans rien modifier que l'état restauré est compatible avec l'espèce.
    pub(crate) fn verify_species(&self, species: &SpeciesDefinition) -> Result<(), String> {
        if species.id() != self.species_id {
            return Err("Définition d'espèce absente du catalogue.".into());
        }
        self.genome.validate()?;
        if !self.birth.is_valid() || !self.levels.is_valid() {
            return Err("Expression des gènes invalide.".into());
        }
        let capacity = Morphology::develop(
            self.genome.express(&self.birth),
            species.rules().energy_capacity(),
        )
        .capacity();
        if !self.energy.is_finite()
            || self.energy <= 0.0
            || self.energy > capacity * (1.0 + 1e-4)
            || self.last_division > self.age
        {
            return Err("État physiologique invalide.".into());
        }
        Ok(())
    }

    /// Adopte une espèce déjà vérifiée par [`Cell::verify_species`] et reconstruit forme et
    /// fonctionnement, qui ne sont jamais sauvegardés.
    pub(crate) fn adopt_species(&mut self, species: Arc<SpeciesDefinition>) {
        self.species = species;
        self.develop_body();
    }

    /// Convertit un organisme d'un ancien fichier : son génome est reconstruit par l'appelant.
    pub(crate) fn from_legacy(
        id: Uuid,
        x: usize,
        y: usize,
        energy: f32,
        genome: Genome,
        age: Duration,
        last_division: Duration,
        movement_elapsed: Duration,
        species: Arc<SpeciesDefinition>,
    ) -> Self {
        let mut cell = Self::born(x, y, genome, Signals::default(), species);
        cell.id = id;
        cell.energy = energy.min(cell.capacity());
        cell.age = age;
        cell.last_division = last_division;
        cell.movement_elapsed = movement_elapsed;
        cell
    }

    pub(crate) fn x(&self) -> usize {
        self.x
    }

    pub(crate) fn y(&self) -> usize {
        self.y
    }

    /// Position réelle en unités de site : `(x + est, y + sud)`, dans `[0, taille)`.
    pub(crate) fn position(&self) -> (f32, f32) {
        (self.x as f32 + self.east, self.y as f32 + self.south)
    }

    /// Fractions de position dans le site, pour la persistance.
    pub(crate) fn offsets(&self) -> (f32, f32) {
        (self.east, self.south)
    }

    /// Contrôle des fractions lues dans une sauvegarde.
    pub(crate) fn offsets_are_valid(east: f32, south: f32) -> bool {
        (0.0..1.0).contains(&east) && (0.0..1.0).contains(&south)
    }

    pub(crate) fn restore_offsets(&mut self, east: f32, south: f32) {
        self.east = east;
        self.south = south;
    }

    /// Sauvegardes sans positions fines : le point dans le site est dérivé de l'identité, ce
    /// qui évite d'empiler tous les organismes d'un site sur le même point.
    pub(crate) fn settle_legacy_offsets(&mut self) {
        let bytes = self.id.as_bytes();
        let fraction = |a: u8, b: u8| 0.15 + 0.7 * (u16::from_le_bytes([a, b]) as f32 / 65_536.0);
        self.east = fraction(bytes[10], bytes[11]);
        self.south = fraction(bytes[14], bytes[15]);
    }

    /// Pose l'organisme en `point` (unités de site), avec bouclage du monde torique.
    fn set_position(&mut self, point: (f32, f32), size: usize) {
        let extent = size as f32;
        let split = |value: f32| {
            let wrapped = value.rem_euclid(extent);
            let site = (wrapped.floor() as usize).min(size - 1);
            (site, (wrapped - site as f32).clamp(0.0, 0.999_999))
        };
        (self.x, self.east) = split(point.0);
        (self.y, self.south) = split(point.1);
    }

    /// Installe un nouveau-né à `point` sans coût énergétique.
    pub(crate) fn settle(&mut self, point: (f32, f32), size: usize) {
        self.set_position(point, size);
    }

    pub(crate) fn energy(&self) -> f32 {
        self.energy
    }

    pub(crate) fn age(&self) -> Duration {
        self.age
    }

    pub(crate) fn last_division(&self) -> Duration {
        self.last_division
    }

    /// Part de l'énergie manquante que l'organisme tente d'ingérer.
    pub(crate) fn ingestion(&self) -> f32 {
        self.physiology().ingestion()
    }

    /// Part de sa réserve de lumière que cet organisme réclame sur son site.
    pub(crate) fn photosynthesis(&self) -> f32 {
        self.physiology().photosynthesis()
    }

    pub(crate) fn thermosynthesis(&self) -> f32 {
        self.physiology().thermosynthesis()
    }

    pub(crate) fn predation(&self) -> f32 {
        self.physiology().predation()
    }

    pub(crate) fn adhesion(&self) -> f32 {
        self.physiology().adhesion()
    }

    pub(crate) fn motility(&self) -> f32 {
        self.physiology().motility()
    }

    /// Un organisme sessile ne se déplace jamais de lui-même (il subit toujours les courants).
    pub(crate) fn is_sessile(&self) -> bool {
        self.motility() < SESSILE_MOTILITY
    }

    /// Vrai si les gènes se réexpriment pendant ce pas (une fois par seconde de vie).
    pub(crate) fn will_refresh(&self, dt: Duration) -> bool {
        (self.age + dt).as_secs() != self.age.as_secs()
    }

    /// Vrai si le réseau neuronal de l'organisme lit la lumière à chaque pas.
    pub(crate) fn senses_light(&self) -> bool {
        self.morphology().neural_units() > 0 && self.genome.sensor() == 1
    }

    /// Vrai si le réseau neuronal de l'organisme lit la température à chaque pas.
    pub(crate) fn senses_temperature(&self) -> bool {
        self.morphology().neural_units() > 0 && self.genome.sensor() == 0
    }

    /// Capteur lu par le réseau neuronal, d'après le génome.
    pub(crate) fn sensor(&self) -> u8 {
        self.genome.sensor()
    }

    /// Énergie que l'organisme capte par seconde dans `conditions`, hors ingestion.
    pub(crate) fn harvest_rate(&self, conditions: &Conditions) -> f32 {
        self.photosynthesis() * PHOTOSYNTHESIS_YIELD * conditions.light()
            + self.thermosynthesis()
                * THERMOSYNTHESIS_YIELD
                * (conditions.warmth().clamp(0.0, THERMOSYNTHESIS_SATURATION) / 10.0)
    }

    /// Valeur lue par le capteur du réseau neuronal.
    fn sense(&self, conditions: &Conditions) -> f32 {
        match self.genome.sensor() {
            0 => conditions.temperature(),
            1 => conditions.light() * 40.0,
            2 => conditions.food_available() * 4.0,
            _ => self.energy,
        }
    }

    /// Signaux que les gènes lisent maintenant.
    fn signals(&self, conditions: &Conditions) -> Signals {
        conditions.signals(
            self.energy / self.capacity(),
            self.age.as_secs_f32() / self.species.rules().maturity_seconds() as f32,
        )
    }

    pub(crate) fn update(&mut self, conditions: &Conditions, dt: Duration) -> bool {
        // Les gènes se réexpriment à chaque seconde de vie : la physiologie suit le milieu.
        let before = self.age.as_secs();
        self.age += dt;
        if self.age.as_secs() != before {
            self.levels = self.genome.express(&self.signals(conditions));
            self.physiology = Some(Physiology::express(&self.levels, self.morphology(), self.genome.genes().len()));
        }

        let seconds = dt.as_secs_f32();
        let upkeep = self.maintenance_cost(conditions.environment(), dt);
        let harvest = self.harvest_rate(conditions);
        // Les dépenses et les gains sont des taux par seconde de temps simulé.
        self.energy = (self.energy - upkeep + harvest * seconds).min(self.capacity());

        // La recherche de nourriture est indépendante de la température et du rendu.
        // Un déplacement coûte une seconde d'activité métabolique et ne peut survenir qu'une fois/seconde.
        self.movement_elapsed += dt;
        self.activation = self
            .genome
            .neural_activation(self.morphology().neural_units(), self.sense(conditions));
        let neural_movement = self.activation > 0.0;
        // Un organisme qui capte de quoi couvrir son entretien n'a pas à chercher plus loin.
        let self_sufficient = seconds > 0.0 && harvest * seconds >= upkeep;
        let wants_to_move = !self.is_sessile()
            && !self_sufficient
            && self.energy > self.species.rules().movement_reserve()
            && conditions.food_available() * self.ingestion()
                < self.energy_deficit().min(self.physiology().metabolism_rate())
            && (self.energy < self.division_threshold() || neural_movement);
        // Une motilité faible espace les décisions de mouvement.
        let interval = Duration::from_secs_f32(
            self.species.rules().movement_seconds() / self.motility().max(SESSILE_MOTILITY),
        );
        if self.movement_elapsed >= interval {
            self.movement_elapsed -= interval;
            return wants_to_move;
        }
        false
    }

    pub(crate) fn maintenance_cost(&self, environment: Environment, dt: Duration) -> f32 {
        let physiology = self.physiology();
        let penalty = match environment {
            Environment::Water => 1.0 - physiology.water_adaptation(),
            Environment::Land => 1.0 - physiology.land_adaptation(),
        };
        // Chaque produit et chaque unité nerveuse a un entretien ; la taille fait croître la
        // dépense comme la masse à la puissance 3/4 ; l'économie de l'ancêtre est la référence.
        physiology.metabolism_rate()
            * (1.0 + penalty + physiology.machinery()).max(0.2)
            * physiology.upkeep_scale()
            * dt.as_secs_f32()
    }

    /// Énergie que ce prédateur prélève sur une proie pendant `dt`, avant rendement.
    pub(crate) fn bite(&self, dt: Duration) -> f32 {
        self.predation() * PREDATION_RATE * dt.as_secs_f32()
    }

    /// Reçoit l'énergie arrachée à une proie, avec le rendement de la prédation.
    pub(crate) fn digest(&mut self, taken: f32) {
        self.feed(taken * PREDATION_EFFICIENCY);
    }

    /// Prélève jusqu'à `wanted` d'énergie sur cet organisme et renvoie la quantité prise.
    pub(crate) fn be_bitten(&mut self, wanted: f32) -> f32 {
        let taken = wanted.max(0.0).min(self.energy.max(0.0));
        self.energy -= taken;
        taken
    }

    /// Reçoit ou cède de l'énergie à un organisme lié : la réserve reste dans `[0, capacité]`.
    pub(crate) fn exchange(&mut self, amount: f32) {
        self.energy = (self.energy + amount).clamp(0.0, self.capacity());
    }

    /// Foudre et autres chocs : retire de l'énergie, jamais plus qu'il n'en reste.
    pub(crate) fn damage(&mut self, amount: f32) {
        self.energy -= amount.max(0.0).min(self.energy.max(0.0));
    }

    /// Poussée maximale de nage contre le courant, en sites par seconde.
    pub(crate) fn swim_capacity(&self) -> f32 {
        self.physiology().swim_speed()
    }

    /// Subit le courant de son site pendant `dt`. L'organisme nage contre l'écoulement avec toute
    /// sa poussée disponible (cela lui coûte de l'énergie) ; le reste du courant l'emporte. Une
    /// lame d'eau mince ne l'entraîne que partiellement. Renvoie le déplacement (est, sud) en sites.
    pub(crate) fn swim_against(&mut self, flow: Flow, dt: Duration) -> Option<(f32, f32)> {
        let wetted = flow.wetted();
        let speed = flow.speed() * wetted;
        if speed < 1e-4 {
            return None;
        }
        let capacity = self.swim_capacity();
        let held = speed.min(capacity);
        if capacity > 0.0 {
            self.energy -= self.physiology().metabolism_rate()
                * SWIM_COST
                * (held / capacity)
                * dt.as_secs_f32();
        }
        let carried = (speed - held) * dt.as_secs_f32() / speed;
        (carried > 0.0).then(|| (flow.east() * wetted * carried, flow.south() * wetted * carried))
    }

    /// Division : la fille reçoit un génome muté dont le taux dépend de la réparation de la mère et
    /// de l'`exposure` du site (rayonnement, chaleur). Elle naît avec la moitié de l'énergie ; sa
    /// forme se développe à son arrivée (voir [`Cell::develop`]), car elle dépend du milieu où elle naît.
    pub(crate) fn try_divide(&mut self, rng: &mut impl rand::Rng, exposure: f32) -> Option<Cell> {
        if self.energy < self.division_threshold()
            || self.age - self.last_division
                < Duration::from_secs(self.species.rules().maturity_seconds())
        {
            return None;
        }
        let genome = self
            .genome
            .reproduce(rng, self.physiology().repair(), exposure);

        // La cellule parent perd de l'énergie
        self.energy /= 2.0;
        self.last_division = self.age;

        let mut daughter = Cell::born(self.x, self.y, genome, self.birth, self.species.clone());
        daughter.east = self.east;
        daughter.south = self.south;
        daughter.energy = self.energy.min(daughter.capacity());
        Some(daughter)
    }

    /// Développe le corps d'une nouvelle-née d'après les signaux du milieu où elle naît.
    pub(crate) fn develop(&mut self, birth: Signals) {
        self.birth = birth;
        self.levels = self.genome.express(&birth);
        self.develop_body();
        self.energy = self.energy.min(self.capacity());
    }

    /// Déplacement actif vers `point` (unités de site) : il coûte le même effort quelle que soit
    /// la direction.
    pub(crate) fn move_to(&mut self, point: (f32, f32), size: usize) {
        if point == self.position() {
            return;
        }
        self.energy -=
            self.physiology().metabolism_rate() * self.species.rules().movement_energy_seconds();
        self.set_position(point, size);
    }

    pub(crate) fn energy_deficit(&self) -> f32 {
        (self.capacity() - self.energy).max(0.0)
    }

    pub(crate) fn feed(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(self.capacity());
    }
}
