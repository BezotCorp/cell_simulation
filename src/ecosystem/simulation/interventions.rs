use super::Simulation;
use crate::biology::Cell;
use crate::definitions::{Definitions, FounderDefinition};
use crate::ecosystem::{Intervention, KillTarget, ResourceKind};
use crate::habitat::{Entity, Mineral};
use rand::RngExt;
use std::sync::Arc;

impl Simulation {
    /// Applique une intervention de l'expérimentateur et décrit son effet.
    pub(crate) fn intervene(&mut self, intervention: &Intervention) -> Result<String, String> {
        match intervention {
            Intervention::Spawn { group, at } => self.spawn(group, *at),
            Intervention::Kill(target) => Ok(format!("{} cellule(s) supprimée(s)", self.kill(*target))),
            Intervention::Storm { sites, intensity } => {
                let size = self.grid.size();
                for _ in 0..*sites {
                    let (x, y) = (self.rng.random_range(0..size), self.rng.random_range(0..size));
                    self.grid.hydrology_mut().add_cloud(x, y, 0.5);
                    self.grid.storm_mut().overcharge(x, y, size, *intensity);
                }
                Ok(format!("Orage déclenché sur {sites} site(s)"))
            }
            Intervention::Rain { sites, amount } => {
                let size = self.grid.size();
                for _ in 0..*sites {
                    let (x, y) = (self.rng.random_range(0..size), self.rng.random_range(0..size));
                    self.grid.hydrology_mut().add_cloud(x, y, *amount);
                }
                Ok(format!("Nuages formés sur {sites} site(s)"))
            }
            Intervention::Scatter { resource, count, amount } => {
                let placed = self.scatter(*resource, *count, *amount);
                Ok(format!("{placed} × {} ajouté(s)", resource.label().to_lowercase()))
            }
            Intervention::FillFood => {
                self.grid.fill_food();
                Ok("Réserves alimentaires remplies".into())
            }
        }
    }

    /// Ajoute les cellules d'un groupe ; renvoie un message d'erreur si son génome est invalide.
    fn spawn(&mut self, group: &FounderDefinition, at: Option<(usize, usize)>) -> Result<String, String> {
        let species = self.definitions.species().clone();
        let genome = species.founder_genome(group)?;
        let size = self.grid.size();
        let open: Vec<(usize, usize)> = (0..size)
            .flat_map(|y| (0..size).map(move |x| (x, y)))
            .filter(|&(x, y)| self.grid.is_passable(x, y))
            .collect();
        let start = at.filter(|&(x, y)| x < size && y < size && self.grid.is_passable(x, y)).or(self.grid.initial_position());
        let mut created = 0;
        for _ in 0..group.count {
            let (x, y) = match (group.scatter, start) {
                (true, _) if !open.is_empty() => open[self.rng.random_range(0..open.len())],
                (_, Some(site)) => site,
                _ => break,
            };
            let mut cell = Cell::founder(x, y, genome.clone(), group.energy, species.clone());
            cell.settle((x as f32 + self.rng.random::<f32>(), y as f32 + self.rng.random::<f32>()), size);
            self.lineage.record_birth(cell.id(), None, self.ticks);
            self.cells.push(cell);
            created += 1;
        }
        Ok(format!("{created} cellule(s) créée(s)"))
    }

    /// Supprime les cellules visées, immédiatement, et renvoie leur nombre.
    fn kill(&mut self, target: KillTarget) -> usize {
        let count = self.cells.len();
        let mut doomed: Vec<usize> = match target {
            KillTarget::Everyone => (0..count).collect(),
            KillTarget::Fraction(fraction) => {
                let fraction = fraction.clamp(0.0, 1.0);
                (0..count).filter(|_| self.rng.random::<f32>() < fraction).collect()
            }
            KillTarget::Count(n) => {
                let mut all: Vec<usize> = (0..count).collect();
                let mut chosen = Vec::new();
                for _ in 0..(n as usize).min(count) {
                    chosen.push(all.swap_remove(self.rng.random_range(0..all.len())));
                }
                chosen
            }
            KillTarget::Weakest(n) => {
                let mut order: Vec<usize> = (0..count).collect();
                order.sort_by(|a, b| self.cells[*a].energy().total_cmp(&self.cells[*b].energy()));
                order.truncate(n as usize);
                order
            }
            KillTarget::Around { x, y, radius } => {
                let size = self.grid.size() as f32;
                let wrap = |d: f32| (d + size / 2.0).rem_euclid(size) - size / 2.0;
                (0..count)
                    .filter(|&i| {
                        let (cx, cy) = self.cells[i].position();
                        let (dx, dy) = (wrap(cx - x as f32 - 0.5), wrap(cy - y as f32 - 0.5));
                        dx.hypot(dy) <= radius
                    })
                    .collect()
            }
        };
        doomed.sort_unstable();
        doomed.dedup();
        for &index in &doomed {
            self.cells[index].kill();
        }
        self.bury_dead();
        let lineage = &self.lineage;
        self.bonds.retain(|a, b| lineage.is_alive(a) && lineage.is_alive(b));
        doomed.len()
    }

    /// Pose `count` éléments sur des sites libres au hasard ; renvoie combien ont pu l'être.
    fn scatter(&mut self, resource: ResourceKind, count: u32, amount: f32) -> u32 {
        let size = self.grid.size();
        let capacity = self.definitions.world().food_capacity();
        let mut placed = 0;
        for _ in 0..count.saturating_mul(8) {
            if placed == count {
                break;
            }
            let (x, y) = (self.rng.random_range(0..size), self.rng.random_range(0..size));
            let entity = match resource {
                ResourceKind::Food => Entity::Food(amount.clamp(0.0, capacity)),
                ResourceKind::HeatSource => Entity::HeatSource(amount.max(0.0)),
                ResourceKind::Oxygen => Entity::Oxygen(amount.clamp(0.0, 1.0)),
                ResourceKind::Puddle => Entity::Water(amount.clamp(0.0, 1.0)),
                ResourceKind::Iron => Entity::Minerals(Mineral::Iron),
                ResourceKind::Calcium => Entity::Minerals(Mineral::Calcium),
                ResourceKind::Phosphorus => Entity::Minerals(Mineral::Phosphorus),
            };
            if self.grid.place(x, y, entity) {
                placed += 1;
            }
        }
        placed
    }

    /// Adopte de nouvelles définitions sans repartir de zéro : les cellules gardent leur génome et
    /// se réexpriment avec les nouvelles règles. Les réserves alimentaires sont ramenées à la
    /// nouvelle capacité ; un changement de taille du monde est refusé (il faut redémarrer).
    pub(crate) fn redefine(&mut self, definitions: Arc<Definitions>) -> Result<(), String> {
        if definitions.world().size() != self.grid.size() {
            return Err("La taille du monde ne peut pas changer en direct : redémarrez l'expérience.".into());
        }
        self.grid.clamp_food(definitions.world().food_capacity());
        self.bind_definitions(definitions)
    }
}
