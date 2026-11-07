use crate::ecosystem::Simulation;
use crate::persistence::SessionView;
use crate::world_scene::{
    Picked,
    habitat_scene::{self},
    habitat_visual::HabitatVisual,
    materials::{Gradient, Materials},
    organism_visual::OrganismVisual,
};
use crate::{
    definitions::MaterialKind, habitat::Entity, habitat::Environment, persistence::ViewMode,
};
use fyrox::graph::SceneGraph;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        color::Color,
        pool::Handle,
    },
    resource::texture::{TextureResource, TextureResourceExtension},
    scene::{
        Scene,
        base::BaseBuilder,
        camera::{Camera, CameraBuilder},
        light::{BaseLightBuilder, directional::DirectionalLightBuilder, point::PointLightBuilder},
        mesh::Mesh,
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
/// Nombre de paliers de hauteur de la nourriture.
const FOOD_LEVELS: f32 = 32.0;

pub(crate) struct WorldScene {
    scene: Handle<Scene>,
    camera: Handle<Camera>,
    target: TextureResource,
    render_size: (u32, u32),
    resize_request: Option<((u32, u32), Instant)>,
    materials: Materials,
    habitats: Vec<HabitatVisual>,
    cells: HashMap<Uuid, OrganismVisual>,
    selection: Handle<Mesh>,
    selected: Option<Uuid>,
    size: usize,
    spacing: f32,
    /// Dernier état logique reflété dans la scène : (pas biologique, vue, première synchro faite).
    synced: Option<(u64, ViewMode)>,
    /// Occupants (habitat, identité, position dans la population), tampon réutilisé.
    occupants: Vec<(usize, Uuid, usize)>,
    elapsed: f32,
}

impl WorldScene {
    pub(crate) fn build(
        simulation: &Simulation,
        scenes: &mut fyrox::scene::SceneContainer,
        render_size: (u32, u32),
    ) -> Self {
        let defs = simulation.definitions();
        let materials = Materials::new(defs.appearance());
        let size = simulation.grid().size();
        let spacing = defs.appearance().habitat_spacing();
        let extent = size as f32 * spacing;
        let mut scene = Scene::new();
        scene.set_skybox(None);
        scene.rendering_options.clear_color = Some(Color::opaque(10, 19, 27));
        scene.rendering_options.environment_lighting_source =
            fyrox::scene::EnvironmentLightingSource::AmbientColor;
        let target = TextureResource::new_render_target(render_size.0, render_size.1);
        scene.rendering_options.render_target = Some(target.clone());
        scene.rendering_options.ambient_lighting_color = Color::opaque(86, 106, 120);
        let camera = CameraBuilder::new(
            BaseBuilder::new()
                .with_name("Caméra orbitale")
                .with_script(crate::world_scene::OrbitCamera),
        )
        .with_fov(48.0_f32.to_radians())
        .with_z_near(0.05)
        .with_z_far(extent * 6.0 + 400.0)
        .build(&mut scene.graph);
        DirectionalLightBuilder::new(
            BaseLightBuilder::new(
                BaseBuilder::new()
                    .with_name("Lumière principale")
                    .with_local_transform(
                        TransformBuilder::new()
                            .with_local_rotation(UnitQuaternion::from_euler_angles(-0.8, -0.5, 0.0))
                            .build(),
                    ),
            )
            .with_color(Color::opaque(255, 232, 204))
            .with_scatter_enabled(false)
            .with_intensity(1.7),
        )
        .build(&mut scene.graph);
        // Contre-jour froid proportionnel à la taille du biotope, placé hors du plan de la grille.
        PointLightBuilder::new(
            BaseLightBuilder::new(
                BaseBuilder::new()
                    .with_name("Lumière froide")
                    .with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                -extent * 0.45,
                                extent * 0.55 + 2.0,
                                -extent * 0.35,
                            ))
                            .build(),
                    ),
            )
            .with_color(Color::opaque(90, 174, 237))
            .with_scatter_enabled(false)
            .with_intensity(5.0),
        )
        .with_radius(extent * 2.2 + 8.0)
        .build(&mut scene.graph);
        crate::world_scene::geometry::mesh(
            &mut scene.graph,
            "Socle du biotope",
            Uuid::now_v7(),
            Vector3::new(0.0, -0.55, 0.0),
            Vector3::new(extent + 0.7, 0.65, extent + 0.7),
            false,
            materials.get(MaterialKind::Plinth),
        );
        for (position, scale) in [
            (
                Vector3::new(0.0, -0.18, -extent / 2.0 - 0.16),
                Vector3::new(extent + 0.5, 0.16, 0.16),
            ),
            (
                Vector3::new(0.0, -0.18, extent / 2.0 + 0.16),
                Vector3::new(extent + 0.5, 0.16, 0.16),
            ),
            (
                Vector3::new(-extent / 2.0 - 0.16, -0.18, 0.0),
                Vector3::new(0.16, 0.16, extent + 0.5),
            ),
            (
                Vector3::new(extent / 2.0 + 0.16, -0.18, 0.0),
                Vector3::new(0.16, 0.16, extent + 0.5),
            ),
        ] {
            crate::world_scene::geometry::mesh(
                &mut scene.graph,
                "Cadre",
                Uuid::now_v7(),
                position,
                scale,
                false,
                materials.get(MaterialKind::Rim),
            );
        }
        let grid = simulation.grid();
        let mut habitats = Vec::with_capacity(size * size);
        let mut heat_sources = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let (wx, wz) = Self::tile_center(size, spacing, x, y);
                let entity = grid.get_entity(x, y);
                let tile = habitat_scene::tile(
                    &mut scene.graph,
                    grid.habitat_id(x, y),
                    wx,
                    wz,
                    spacing,
                    &materials,
                );
                // Les habitats vides n'ont pas d'objet : il est créé si une ressource y apparaît.
                let object = (!matches!(entity, Entity::Empty)).then(|| {
                    habitat_scene::object(&mut scene.graph, entity, wx, wz, spacing, &materials)
                });
                habitats.push(HabitatVisual::new(tile, object));
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
                                .with_local_position(Vector3::new(wx, 0.5, wz))
                                .build(),
                        ),
                )
                .with_color(Color::opaque(242, 117, 53))
                .with_scatter_enabled(false)
                .with_intensity(0.9),
            )
            .with_radius(2.4 * spacing)
            .build(&mut scene.graph);
        }
        let selection = crate::world_scene::geometry::mesh(
            &mut scene.graph,
            "Sélection",
            Uuid::now_v7(),
            Vector3::zeros(),
            Vector3::new(0.1, 0.1, 0.1),
            true,
            materials.get(MaterialKind::Selection),
        );
        scene.graph[selection].set_visibility(false);
        Self {
            scene: scenes.add(scene),
            camera,
            target,
            render_size,
            resize_request: None,
            materials,
            habitats,
            cells: HashMap::new(),
            selection,
            selected: None,
            size,
            spacing,
            synced: None,
            occupants: Vec::new(),
            elapsed: 0.0,
        }
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
    /// l'apparence diffère réellement touchent au graphe de scène.
    pub(crate) fn sync(&mut self, simulation: &Simulation, view: &SessionView, scene: &mut Scene) {
        let key = (simulation.ticks(), view.mode());
        if self.synced != Some(key) {
            self.sync_habitats(simulation, view.mode(), scene);
            self.sync_organisms(simulation, view.mode(), scene);
            self.synced = Some(key);
        }
        self.selected = view.selected().filter(|id| self.cells.contains_key(id));
        if self.selected.is_none() {
            scene.graph[self.selection].set_visibility(false);
        }
    }

    fn sync_habitats(&mut self, simulation: &Simulation, mode: ViewMode, scene: &mut Scene) {
        let grid = simulation.grid();
        for (index, visual) in self.habitats.iter_mut().enumerate() {
            let (x, y) = (index % self.size, index / self.size);
            let state = habitat_scene::tile_state(
                mode,
                grid.get_temperature(x, y),
                grid.get_environment_type(x, y) == Environment::Water,
            );
            if visual.update_tile_state(state) {
                scene.graph[visual.tile()].surfaces_mut()[0]
                    .set_material(habitat_scene::tile_material(state, &self.materials));
            }
        }
        let capacity = simulation.definitions().world().food_capacity();
        for &index in grid.food_sites() {
            let (x, y) = (index % self.size, index / self.size);
            let (wx, wz) = Self::tile_center(self.size, self.spacing, x, y);
            let visual = &mut self.habitats[index];
            let Entity::Food(energy) = grid.get_entity(x, y) else {
                continue;
            };
            // Un site renouvelable peut apparaître après la création de la scène.
            let object = match visual.object() {
                Some(object) => object,
                None => {
                    let object = habitat_scene::object(
                        &mut scene.graph,
                        grid.get_entity(x, y),
                        wx,
                        wz,
                        self.spacing,
                        &self.materials,
                    );
                    visual.set_object(object);
                    object
                }
            };
            let fill = (energy / capacity).clamp(0.0, 1.0);
            let bucket = (fill * FOOD_LEVELS).round() as u8;
            if visual.update_food_bucket(bucket) {
                let height = 0.06 + 0.13 * bucket as f32 / FOOD_LEVELS;
                scene.graph[object]
                    .local_transform_mut()
                    .set_scale(Vector3::new(
                        0.20 * self.spacing,
                        height,
                        0.20 * self.spacing,
                    ))
                    .set_position(Vector3::new(wx, height * 0.5, wz));
            }
        }
    }

    fn sync_organisms(&mut self, simulation: &Simulation, mode: ViewMode, scene: &mut Scene) {
        self.cells.retain(|id, visual| {
            let alive = simulation.is_alive(*id);
            if !alive {
                scene.graph.remove_node(visual.body());
            }
            alive
        });
        let grow = self.synced.is_some();
        let grid = simulation.grid();
        let base_radius = simulation.definitions().appearance().cell_radius();
        let capacity = simulation.definitions().species().rules().energy_capacity();

        self.occupants.clear();
        self.occupants.extend(
            simulation
                .cells()
                .iter()
                .enumerate()
                .map(|(position, cell)| (cell.y() * self.size + cell.x(), cell.id(), position)),
        );
        self.occupants
            .sort_unstable_by_key(|&(habitat, id, _)| (habitat, id));

        let mut start = 0;
        while start < self.occupants.len() {
            let habitat = self.occupants[start].0;
            let count = self.occupants[start..]
                .iter()
                .take_while(|occupant| occupant.0 == habitat)
                .count();
            let (x, y) = (habitat % self.size, habitat / self.size);
            let (wx, wz) = Self::tile_center(self.size, self.spacing, x, y);
            let temperature = grid.get_temperature(x, y);

            for (slot, &(_, id, position)) in
                self.occupants[start..start + count].iter().enumerate()
            {
                let cell = &simulation.cells()[position];
                let (offset, fit) = OrganismVisual::slot(slot, count, self.spacing);
                let radius = base_radius.min(fit);
                let energy = (cell.energy() / capacity).clamp(0.0, 1.0);
                let displayed = radius * (0.85 + 0.15 * energy);
                let target = Vector3::new(wx + offset.x, displayed + 0.03, wz + offset.z);

                let visual = self.cells.entry(id).or_insert_with(|| {
                    OrganismVisual::new(
                        &mut scene.graph,
                        cell,
                        if grow { displayed * 0.3 } else { displayed },
                        target,
                        &self.materials,
                    )
                });
                visual.set_target(target, displayed, self.spacing * 2.0);

                let (gradient, value) = match mode {
                    ViewMode::Ecosystem => (Gradient::Energy, energy),
                    ViewMode::Temperature => (Gradient::Temperature, (temperature - 15.0) / 45.0),
                    ViewMode::Adaptation => (Gradient::Adaptation, cell.genes().water_adaptation()),
                };
                let band = Materials::band(value);
                if visual.update_material(band, mode as u8) {
                    scene.graph[visual.membrane()].surfaces_mut()[0]
                        .set_material(self.materials.gradient(gradient, band));
                }
            }
            start += count;
        }
    }

    /// Anime l'affichage vers l'état biologique, en secondes réelles : la fluidité ne dépend
    /// ni de la cadence d'affichage ni de la vitesse de simulation.
    pub(crate) fn animate(&mut self, dt: f32, scene: &mut Scene) {
        self.elapsed += dt;
        for visual in self.cells.values_mut() {
            if visual.animate(dt) {
                scene.graph[visual.body()]
                    .local_transform_mut()
                    .set_position(visual.position());
                scene.graph[visual.membrane()]
                    .local_transform_mut()
                    .set_scale(Vector3::repeat(visual.radius()));
            }
        }
        if let Some(visual) = self.selected.and_then(|id| self.cells.get(&id)) {
            let bob = 0.03 * (self.elapsed * 3.0).sin();
            scene.graph[self.selection].set_visibility(true);
            scene.graph[self.selection]
                .local_transform_mut()
                .set_position(
                    visual.position() + Vector3::new(0.0, visual.radius() * 1.9 + 0.1 + bob, 0.0),
                )
                .set_scale(Vector3::repeat(visual.radius().max(0.05) * 0.26));
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
        let ray = scene.graph[self.camera].make_ray(point, self.image_size());

        let organism = self
            .cells
            .iter()
            .filter_map(|(id, visual)| {
                let to_center = visual.position() - ray.origin;
                let along = to_center.dot(&ray.dir);
                let radius = visual.radius().max(0.04 * self.spacing);
                let miss_squared = to_center.norm_squared() - along * along;
                let depth_squared = radius * radius - miss_squared;
                (depth_squared >= 0.0).then(|| (along - depth_squared.sqrt(), *id))
            })
            .filter(|(distance, _)| *distance >= 0.0)
            .min_by(|a, b| a.0.total_cmp(&b.0));

        let habitat = (ray.dir.y < -1e-6)
            .then(|| -ray.origin.y / ray.dir.y)
            .filter(|distance| *distance >= 0.0)
            .and_then(|distance| {
                let hit = ray.origin + ray.dir * distance;
                let offset = (self.size as f32 - 1.0) / 2.0;
                let (fx, fz) = (hit.x / self.spacing + offset, hit.z / self.spacing + offset);
                let (ix, iz) = (fx.round(), fz.round());
                // Même emprise que la dalle visible : 0,48 de la maille autour de son centre.
                (ix >= 0.0
                    && iz >= 0.0
                    && (ix as usize) < self.size
                    && (iz as usize) < self.size
                    && (fx - ix).abs() <= 0.48
                    && (fz - iz).abs() <= 0.48)
                    .then_some((distance, iz as usize * self.size + ix as usize))
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
