use crate::ecosystem::Simulation;
use crate::persistence::SessionView;
use crate::world_scene::{
    OrbitCamera, Picked,
    geometry::{self, mesh},
    gradient::Gradient,
    ground::Ground,
    habitat_animation::HabitatAnimation,
    habitat_props, lighting,
    habitat_visual::HabitatVisual,
    materials::Materials,
    organism_visual::OrganismVisual,
    selection_marker::SelectionMarker,
    surfaces::Surfaces,
    bond_links::BondLinks,
    component_library::ComponentLibrary,
    detail_level::DetailLevel,
    flow_streaks::FlowStreaks,
    lightning_show::LightningShow,
    rain_shower::RainShower,
    water_overlay::WaterOverlay,
};
use crate::{definitions::MaterialKind, habitat::Entity, persistence::ViewMode};
use fyrox::graph::SceneGraph;
use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        color::Color,
        pool::Handle,
    },
    resource::texture::{TextureResource, TextureResourceExtension},
    scene::{
        Scene,
        base::BaseBuilder,
        camera::{Camera, CameraBuilder},
        light::{BaseLightBuilder, point::PointLightBuilder},
        transform::TransformBuilder,
    },
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Nombre maximal de lumières dynamiques associées aux sources thermiques. Les sources
/// supplémentaires restent visibles (matériau émissif) mais n'éclairent pas la scène.
const HEAT_LIGHTS: usize = 12;
/// Délai de stabilité avant de recréer la cible de rendu après un redimensionnement.
const RESIZE_SETTLE: Duration = Duration::from_millis(120);
/// Nombre de paliers de remplissage de la nourriture.
const FOOD_LEVELS: f32 = 32.0;
/// Au-delà de ce nombre d'organismes, les organelles ne sont plus affichées : la foule reste
/// lisible et le rendu fluide. Aucun individu n'est retiré, seule la finesse de son dessin change.
const DETAIL_LIMIT: usize = 2_500;
/// En deçà de ce nombre d'organismes, leur anatomie est dessinée en entier ; au-delà, seuls le noyau
/// et les deux composants les plus développés le sont.
const FULL_DETAIL_LIMIT: usize = 300;
/// Au-delà de ce nombre d'organismes, ils ne projettent plus d'ombre propre.
const SHADOW_LIMIT: usize = 2_000;
/// Au-delà de ce nombre d'habitats garnis, chaque ressource se réduit à un seul élément sans ombre.
const PROP_DETAIL_LIMIT: usize = 1_200;
/// Nombre maximal de ressources animées en continu (sources thermiques, bulles).
const ANIMATED_LIMIT: usize = 256;

pub(crate) struct WorldScene {
    scene: Handle<Scene>,
    camera: Handle<Camera>,
    target: TextureResource,
    render_size: (u32, u32),
    resize_request: Option<((u32, u32), Instant)>,
    materials: Materials,
    surfaces: Surfaces,
    ground: Ground,
    water: WaterOverlay,
    rain: RainShower,
    streaks: FlowStreaks,
    library: ComponentLibrary,
    links: BondLinks,
    lightning: LightningShow,
    /// Liaisons à dessiner et dernier éclair déjà montré (seconde simulée).
    bond_pairs: Vec<(Uuid, Uuid)>,
    last_strike: Option<u64>,
    /// Vue courante, pour masquer pluie et courants dans la vue thermique.
    mode: ViewMode,
    habitats: HashMap<usize, HabitatVisual>,
    animated: Vec<usize>,
    cells: HashMap<Uuid, OrganismVisual>,
    dying: Vec<OrganismVisual>,
    marker: SelectionMarker,
    selected: Option<Uuid>,
    selected_habitat: Option<usize>,
    size: usize,
    spacing: f32,
    /// Dernier état logique reflété dans la scène : (pas biologique, vue).
    synced: Option<(u64, ViewMode)>,
    /// Occupants (habitat, identité, position dans la population), tampon réutilisé.
    /// Nombre d'organismes par site de la grille technique (densité d'affichage uniquement).
    crowding: Vec<u32>,
    elapsed: f32,
}

impl WorldScene {
    pub(crate) fn build(
        simulation: &Simulation,
        scenes: &mut fyrox::scene::SceneContainer,
        render_size: (u32, u32),
        library: ComponentLibrary,
    ) -> Self {
        let (mut world, scene) = Self::compose(simulation, render_size, library);
        world.scene = scenes.add(scene);
        world
    }

    /// Compose la scène et son état de suivi ; le moteur n'ajoute la scène qu'ensuite.
    pub(crate) fn compose(
        simulation: &Simulation,
        render_size: (u32, u32),
        library: ComponentLibrary,
    ) -> (Self, Scene) {
        let defs = simulation.definitions();
        let mut materials = Materials::new(defs.appearance());
        let surfaces = Surfaces::new();
        let size = simulation.grid().size();
        let spacing = defs.appearance().habitat_spacing();
        let extent = size as f32 * spacing;
        let mut scene = Scene::new();
        scene.set_skybox(None);
        scene.rendering_options.clear_color = Some(Color::opaque(18, 32, 46));
        scene.rendering_options.environment_lighting_source =
            fyrox::scene::EnvironmentLightingSource::AmbientColor;
        scene.rendering_options.ambient_lighting_color = Color::opaque(110, 130, 150);
        let target = TextureResource::new_render_target(render_size.0, render_size.1);
        scene.rendering_options.render_target = Some(target.clone());
        let camera = CameraBuilder::new(
            BaseBuilder::new()
                .with_name("Caméra orbitale")
                .with_script(OrbitCamera),
        )
        .with_projection(OrbitCamera::projection(extent * 6.0 + 400.0))
        .build(&mut scene.graph);
        lighting::add(&mut scene.graph, extent);

        // Plinthe : sa face supérieure affleure le sol ; le cadre en souligne le contour.
        mesh(
            &mut scene.graph,
            "Socle du biotope",
            Uuid::now_v7(),
            Vector3::new(0.0, -0.36, 0.0),
            Vector3::new(extent + 0.7, 0.70, extent + 0.7),
            geometry::cube(),
            materials.get(MaterialKind::Plinth),
        );
        for (position, scale) in [
            (
                Vector3::new(0.0, 0.0, -extent / 2.0 - 0.1),
                Vector3::new(extent + 0.4, 0.14, 0.2),
            ),
            (
                Vector3::new(0.0, 0.0, extent / 2.0 + 0.1),
                Vector3::new(extent + 0.4, 0.14, 0.2),
            ),
            (
                Vector3::new(-extent / 2.0 - 0.1, 0.0, 0.0),
                Vector3::new(0.2, 0.14, extent),
            ),
            (
                Vector3::new(extent / 2.0 + 0.1, 0.0, 0.0),
                Vector3::new(0.2, 0.14, extent),
            ),
        ] {
            mesh(
                &mut scene.graph,
                "Cadre",
                Uuid::now_v7(),
                position,
                scale,
                geometry::cube(),
                materials.get(MaterialKind::Rim),
            );
        }

        let ground = Ground::new(&mut scene.graph, simulation, &mut materials, &surfaces);
        let mut water = WaterOverlay::new(&mut scene.graph, size, extent, spacing, &surfaces);
        water.refresh(simulation.grid().hydrology(), true);
        let mut rain = RainShower::new(&mut scene.graph, &materials, size, spacing);
        rain.refresh(simulation.grid().hydrology());
        let mut streaks = FlowStreaks::new(&mut scene.graph, &materials, size, spacing);
        streaks.refresh(simulation.grid().hydrology());
        let links = BondLinks::new(&mut scene.graph, &materials, spacing);
        let lightning = LightningShow::new(
            &mut scene.graph,
            &materials,
            &surfaces,
            spacing,
            2.6 * spacing,
        );
        let grid = simulation.grid();
        let furnished = (0..size * size)
            .filter(|index| !matches!(grid.get_entity(index % size, index / size), Entity::Empty))
            .count();
        let prop_detail = furnished <= PROP_DETAIL_LIMIT;
        let mut habitats = HashMap::new();
        let mut animated = Vec::new();
        let mut heat_sources = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let (wx, wz) = Self::tile_center(size, spacing, x, y);
                let entity = grid.get_entity(x, y);
                if let Some(visual) = habitat_props::build(
                    &mut scene.graph,
                    entity,
                    (wx, wz),
                    (x, y),
                    spacing,
                    &materials,
                    &surfaces,
                    prop_detail,
                ) {
                    if visual.animation() != HabitatAnimation::Still
                        && animated.len() < ANIMATED_LIMIT
                    {
                        animated.push(y * size + x);
                    }
                    habitats.insert(y * size + x, visual);
                }
                if let Entity::HeatSource(intensity) = entity {
                    heat_sources.push((*intensity, wx, wz));
                }
            }
        }
        heat_sources.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, wx, wz) in heat_sources.into_iter().take(HEAT_LIGHTS) {
            PointLightBuilder::new(
                BaseLightBuilder::new(
                    BaseBuilder::new()
                        .with_name("Source thermique")
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(wx, 0.55 * spacing, wz))
                                .build(),
                        ),
                )
                .with_color(Color::opaque(255, 120, 50))
                .with_scatter_enabled(false)
                .with_intensity(1.4),
            )
            .with_radius(3.2 * spacing)
            .build(&mut scene.graph);
        }
        let marker = SelectionMarker::new(&mut scene.graph, &materials, &surfaces);
        let world = Self {
            scene: Handle::NONE,
            camera,
            target,
            render_size,
            resize_request: None,
            materials,
            surfaces,
            ground,
            water,
            rain,
            streaks,
            library,
            links,
            lightning,
            bond_pairs: Vec::new(),
            last_strike: None,
            mode: ViewMode::Ecosystem,
            habitats,
            animated,
            cells: HashMap::new(),
            dying: Vec::new(),
            marker,
            selected: None,
            selected_habitat: None,
            size,
            spacing,
            synced: None,
            crowding: Vec::new(),
            elapsed: 0.0,
        };
        (world, scene)
    }

    /// Position monde (plan horizontal) d'un point exact exprimé en unités de site.
    fn world_point(size: usize, spacing: f32, point: (f32, f32)) -> (f32, f32) {
        let half = size as f32 / 2.0;
        ((point.0 - half) * spacing, (point.1 - half) * spacing)
    }

    fn tile_center(size: usize, spacing: f32, x: usize, y: usize) -> (f32, f32) {
        let offset = (size as f32 - 1.0) / 2.0;
        ((x as f32 - offset) * spacing, (y as f32 - offset) * spacing)
    }

    /// Adapte la cible de rendu à la zone d'affichage réellement calculée par l'interface.
    /// Pendant un redimensionnement continu la cible n'est recréée qu'une fois la taille
    /// stabilisée, pour ne pas allouer une texture par pixel de déplacement.
    pub(crate) fn resize(&mut self, scene: &mut Scene, area: Vector2<f32>) -> bool {
        // Tant que l'interface n'a pas calculé sa mise en page, la zone est nulle : on attend.
        if area.x < 16.0 || area.y < 16.0 {
            return false;
        }
        let wanted = (area.x as u32, area.y as u32);
        if wanted == self.render_size {
            self.resize_request = None;
            return false;
        }
        let now = Instant::now();
        match self.resize_request {
            Some((size, since)) if size == wanted => {
                if now.duration_since(since) < RESIZE_SETTLE {
                    return false;
                }
            }
            _ => {
                self.resize_request = Some((wanted, now));
                return false;
            }
        }
        self.target = TextureResource::new_render_target(wanted.0, wanted.1);
        scene.rendering_options.render_target = Some(self.target.clone());
        self.render_size = wanted;
        self.resize_request = None;
        true
    }

    fn image_size(&self) -> Vector2<f32> {
        Vector2::new(self.render_size.0 as f32, self.render_size.1 as f32)
    }

    /// Reflète l'état biologique dans la scène. Les parcours coûteux ne sont refaits que
    /// lorsque la simulation a avancé ou que la vue a changé ; seuls les éléments dont
    /// l'apparence diffère réellement touchent au graphe.
    pub(crate) fn sync(&mut self, simulation: &Simulation, view: &SessionView, scene: &mut Scene) {
        let key = (simulation.ticks(), view.mode());
        self.mode = view.mode();
        let forced = self.synced.is_none();
        if self.water.refresh(simulation.grid().hydrology(), forced) {
            self.rain.refresh(simulation.grid().hydrology());
            self.streaks.refresh(simulation.grid().hydrology());
        }
        self.water.show(&mut scene.graph, view.mode());
        self.show_new_strikes(simulation);
        self.bond_pairs.clear();
        self.bond_pairs.extend_from_slice(simulation.bonds().pairs());
        if self.synced != Some(key) {
            self.sync_habitats(simulation, view.mode(), scene);
            self.sync_organisms(simulation, view.mode(), scene);
            self.synced = Some(key);
        }
        self.selected = view.selected().filter(|id| self.cells.contains_key(id));
        self.selected_habitat = view
            .habitat()
            .and_then(|id| simulation.grid().habitat_position(id))
            .map(|(x, y)| y * self.size + x);
        if self.selected.is_none() && self.selected_habitat.is_none() {
            self.marker.hide(&mut scene.graph);
        }
    }

    /// Montre les éclairs que la simulation a produits depuis la dernière synchronisation. Au
    /// premier appel (nouvelle scène, chargement) rien n'est rejoué.
    fn show_new_strikes(&mut self, simulation: &Simulation) {
        let latest = simulation.strikes().iter().map(|strike| strike.second()).max();
        if let Some(seen) = self.last_strike {
            for strike in simulation.strikes().iter().filter(|s| s.second() > seen) {
                let (east, south) = Self::world_point(
                    self.size,
                    self.spacing,
                    (strike.x() as f32 + 0.5, strike.y() as f32 + 0.5),
                );
                self.lightning.strike(
                    east,
                    south,
                    self.elapsed,
                    (strike.second() as u32).wrapping_mul(2_654_435_761) ^ (strike.x() * 31 + strike.y()) as u32,
                );
            }
        }
        self.last_strike = latest.or(self.last_strike).or(Some(0));
    }

    fn sync_habitats(&mut self, simulation: &Simulation, mode: ViewMode, scene: &mut Scene) {
        let grid = simulation.grid();
        self.ground
            .show(&mut scene.graph, mode);
        let capacity = simulation.definitions().world().food_capacity();
        for &index in grid.food_sites() {
            let (x, y) = (index % self.size, index / self.size);
            let Entity::Food(energy) = grid.get_entity(x, y) else {
                continue;
            };
            // Un site renouvelable peut apparaître après la création de la scène.
            if !self.habitats.contains_key(&index) {
                let center = Self::tile_center(self.size, self.spacing, x, y);
                if let Some(visual) = habitat_props::build(
                    &mut scene.graph,
                    grid.get_entity(x, y),
                    center,
                    (x, y),
                    self.spacing,
                    &self.materials,
                    &self.surfaces,
                    self.habitats.len() <= PROP_DETAIL_LIMIT,
                ) {
                    self.habitats.insert(index, visual);
                }
            }
            let Some(visual) = self.habitats.get_mut(&index) else {
                continue;
            };
            let bucket = ((energy / capacity).clamp(0.0, 1.0) * FOOD_LEVELS).round() as u8;
            if visual.update_food_bucket(bucket) {
                visual.set_fill(&mut scene.graph, bucket as f32 / FOOD_LEVELS);
            }
        }
    }

    /// Position d'un organisme sur l'échelle des façons de se nourrir (ingestion, photosynthèse,
    /// thermosynthèse, prédation) : celle de ses produits que son phénotype exprime le plus.
    fn feeding_strategy(cell: &crate::biology::Cell) -> f32 {
        let physiology = cell.physiology();
        let scores = [
            physiology.ingestion(),
            physiology.photosynthesis(),
            physiology.thermosynthesis(),
            physiology.predation(),
        ];
        let best = scores
            .iter()
            .enumerate()
            .fold(0, |best, (index, score)| if *score > scores[best] { index } else { best });
        best as f32 / 3.0
    }

    fn sync_organisms(&mut self, simulation: &Simulation, mode: ViewMode, scene: &mut Scene) {
        let parts = simulation.definitions().species().parts();
        // Les morts rétrécissent avant de disparaître au lieu de s'évanouir d'un coup.
        let departed: Vec<Uuid> = self
            .cells
            .keys()
            .filter(|id| !simulation.is_alive(**id))
            .copied()
            .collect();
        for id in departed {
            if let Some(mut visual) = self.cells.remove(&id) {
                visual.begin_fade();
                self.dying.push(visual);
            }
        }
        let grow = self.synced.is_some();
        let grid = simulation.grid();
        let base_radius = simulation.definitions().appearance().cell_radius();
        let population = simulation.cells().len();

        self.crowding.clear();
        self.crowding.resize(self.size * self.size, 0);
        for cell in simulation.cells() {
            self.crowding[cell.y() * self.size + cell.x()] += 1;
        }

        for cell in simulation.cells() {
            let id = cell.id();
            let point = cell.position();
            let (wx, wz) = Self::world_point(self.size, self.spacing, point);
            let temperature = grid.temperature_at(point);
            let neighbors = self.crowding[cell.y() * self.size + cell.x()] as usize;
            // La taille de l'organisme est celle de sa morphologie ; la foule la réduit un peu.
            let radius = base_radius * cell.morphology().size() * OrganismVisual::crowd_scale(neighbors);
            let energy = (cell.energy() / cell.capacity()).clamp(0.0, 1.0);
            let displayed = radius * (0.85 + 0.15 * energy);
            let target = Vector3::new(wx, displayed + 0.03, wz);
            let detail = if population <= FULL_DETAIL_LIMIT && radius >= base_radius * 0.5 {
                DetailLevel::Full
            } else if population <= DETAIL_LIMIT && radius >= base_radius * 0.5 {
                DetailLevel::Principal
            } else {
                DetailLevel::Bare
            };

            if !self.cells.contains_key(&id) {
                // Une naissance part du parent et grandit jusqu'à sa place.
                let origin = simulation
                    .lineage_record(id)
                    .and_then(|record| record.parent())
                    .and_then(|parent| self.cells.get(&parent))
                    .map_or(target, |parent| parent.position());
                let visual = OrganismVisual::new(
                    &mut scene.graph,
                    cell,
                    &self.surfaces,
                    &self.materials,
                    if grow { displayed * 0.15 } else { displayed },
                    if grow { origin } else { target },
                    population <= SHADOW_LIMIT,
                );
                self.cells.insert(id, visual);
            }
            let visual = self.cells.get_mut(&id).expect("créé ci-dessus");
            visual.set_target(target, displayed, self.spacing * 2.0);
            visual.set_detail(scene, cell, parts, &self.library, &self.surfaces, &self.materials, detail);

            let (gradient, value) = match mode {
                ViewMode::Ecosystem => (Gradient::Energy, energy),
                ViewMode::Temperature => (Gradient::Temperature, (temperature - 15.0) / 45.0),
                ViewMode::Adaptation => (Gradient::Adaptation, cell.physiology().water_adaptation()),
                ViewMode::Strategy => (Gradient::Strategy, Self::feeding_strategy(cell)),
            };
            let band = Materials::band(value);
            if visual.update_material(band, mode as u8) {
                scene.graph[visual.membrane()].surfaces_mut()[0]
                    .set_material(self.materials.gradient(gradient, band));
            }
        }
    }

    /// Anime l'affichage vers l'état biologique, en secondes réelles : la fluidité ne dépend
    /// ni de la cadence d'affichage ni de la vitesse de simulation.
    pub(crate) fn animate(&mut self, dt: f32, scene: &mut Scene) {
        self.elapsed += dt;
        let time = self.elapsed;
        for visual in self.cells.values_mut() {
            if visual.animate(dt) || visual.is_breathing() {
                visual.write(&mut scene.graph, time);
            }
        }
        for visual in &mut self.dying {
            visual.animate(dt);
            visual.write(&mut scene.graph, time);
        }
        self.dying.retain(|visual| {
            if visual.is_gone() {
                scene.graph.remove_node(visual.body());
                false
            } else {
                true
            }
        });
        let weather = self.mode != ViewMode::Temperature;
        self.lightning.animate(&mut scene.graph, time, weather);
        self.links
            .update(&mut scene.graph, &self.bond_pairs, &self.cells, weather);
        self.rain.animate(&mut scene.graph, time, weather);
        self.streaks.animate(&mut scene.graph, dt, weather);
        for index in &self.animated {
            if let Some(visual) = self.habitats.get(index) {
                visual.animate(&mut scene.graph, time);
            }
        }
        if let Some(visual) = self.selected.and_then(|id| self.cells.get(&id)) {
            self.marker
                .show_organism(&mut scene.graph, visual.position(), visual.radius(), time);
        } else if let Some(index) = self.selected_habitat {
            let center = Self::tile_center(self.size, self.spacing, index % self.size, index / self.size);
            self.marker
                .show_site(&mut scene.graph, center, self.spacing, time);
        }
    }

    /// Sélection par lancer de rayon analytique : le plan du biotope donne l'habitat, les
    /// sphères des organismes donnent l'individu ; le plus proche de la caméra l'emporte.
    /// `point` est exprimé en pixels dans la zone d'affichage du monde.
    pub(crate) fn pick(
        &self,
        scene: &Scene,
        simulation: &Simulation,
        point: Vector2<f32>,
    ) -> Option<Picked> {
        // Le rayon du moteur relie les plans proche et lointain : sa direction n'est pas unitaire.
        let ray = scene.graph[self.camera].make_ray(point, self.image_size());
        let direction = ray.dir.try_normalize(1e-9)?;

        let organism = self
            .cells
            .iter()
            .filter_map(|(id, visual)| {
                let to_center = visual.position() - ray.origin;
                let along = to_center.dot(&direction);
                let radius = visual.radius().max(0.04 * self.spacing);
                let miss_squared = to_center.norm_squared() - along * along;
                let depth_squared = radius * radius - miss_squared;
                (depth_squared >= 0.0).then(|| (along - depth_squared.sqrt(), *id))
            })
            .filter(|(distance, _)| *distance >= 0.0)
            .min_by(|a, b| a.0.total_cmp(&b.0));

        let habitat = (direction.y < -1e-6)
            .then(|| -ray.origin.y / direction.y)
            .filter(|distance| *distance >= 0.0)
            .and_then(|distance| {
                let hit = ray.origin + direction * distance;
                // Tout point du biotope appartient à un site : le sol est continu, sans interstice.
                let half = self.size as f32 / 2.0;
                let (fx, fz) = (hit.x / self.spacing + half, hit.z / self.spacing + half);
                let extent = self.size as f32;
                ((0.0..extent).contains(&fx) && (0.0..extent).contains(&fz))
                    .then_some((distance, fz as usize * self.size + fx as usize))
            });

        match (organism, habitat) {
            (Some((near, id)), Some((plane, _))) if near <= plane => Some(Picked::Organism(id)),
            (Some((_, id)), None) => Some(Picked::Organism(id)),
            (_, Some((_, index))) => Some(Picked::Habitat(
                simulation
                    .grid()
                    .habitat_id(index % self.size, index / self.size),
            )),
            (None, None) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn camera_handle(&self) -> Handle<Camera> {
        self.camera
    }

    pub(crate) fn selected_position(&self, id: Uuid) -> Option<[f32; 3]> {
        self.cells.get(&id).map(|v| v.position().into())
    }

    pub(crate) fn scene(&self) -> Handle<Scene> {
        self.scene
    }
    pub(crate) fn target(&self) -> &TextureResource {
        &self.target
    }
}
