use crate::genetics::{Product, REGULATORS, Signal, regulator_label};
use crate::ecosystem::FIXED_STEP;
use crate::habitat::{Entity, Environment};
use crate::application::Pacing;
use crate::ecosystem::PopulationSample;
use crate::interface::{Action, SeriesChart, StatCard};
use crate::persistence::{Session, ViewMode};
use fyrox::graph::SceneGraph;
use fyrox::{
    core::{algebra::Vector2, color::Color, pool::Handle},
    gui::{
        BuildContext, Thickness, UiNode, UserInterface,
        border::BorderBuilder,
        brush::Brush,
        button::{ButtonBuilder, ButtonMessage},
        formatted_text::WrapMode,
        grid::{Column, GridBuilder, GridMessage, Row},
        message::{MessageDirection, UiMessage},
        scroll_viewer::ScrollViewerBuilder,
        stack_panel::StackPanelBuilder,
        text::{TextBuilder, TextMessage},
        text_box::TextBoxBuilder,
        widget::{WidgetBuilder, WidgetMessage},
        wrap_panel::WrapPanelBuilder,
    },
};

const INK: Color = Color::opaque(223, 235, 234);
const MUTED: Color = Color::opaque(141, 166, 174);
const ACCENT: Color = Color::opaque(97, 225, 183);
const GOLD: Color = Color::opaque(249, 211, 118);
const ROSE: Color = Color::opaque(231, 106, 113);
const SKY: Color = Color::opaque(90, 174, 237);

pub(crate) struct Hud {
    /// Facteur d'échelle de l'interface (écrans à forte densité de pixels).
    scale: f32,
    frame: Vector2<f32>,
    root: Handle<UiNode>,
    body: Handle<UiNode>,
    viewport: Handle<UiNode>,
    buttons: Vec<(Handle<UiNode>, Action)>,
    cells: StatCard,
    births: StatCard,
    deaths: StatCard,
    energy: StatCard,
    time: StatCard,
    water: StatCard,
    rain: StatCard,
    sites: StatCard,
    pace: StatCard,
    population_chart: SeriesChart,
    energy_chart: SeriesChart,
    /// Révision de l'historique et largeur de tracé déjà dessinées.
    drawn: Option<(u64, u32)>,
    inspector: Handle<UiNode>,
    status: Handle<UiNode>,
    speed: Handle<UiNode>,
    pause_text: Handle<UiNode>,
    seed: Handle<UiNode>,
    seed_text: String,
    editing_seed: bool,
    shown_inspector: String,
    shown_speed: String,
    shown_pause: Option<bool>,
}

fn text(
    ctx: &mut BuildContext,
    value: &str,
    size: f32,
    color: Color,
    scale: f32,
) -> Handle<UiNode> {
    TextBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(4.0 * scale))
            .with_foreground(Brush::Solid(color).into()),
    )
    .with_text(value)
    .with_font_size((size * scale).into())
    .build(ctx)
    .transmute()
}

fn panel(
    ctx: &mut BuildContext,
    child: Handle<UiNode>,
    widget: WidgetBuilder,
    color: Color,
    scale: f32,
) -> Handle<UiNode> {
    BorderBuilder::new(
        widget
            .with_background(Brush::Solid(color).into())
            .with_child(child),
    )
    .with_corner_radius((7.0 * scale).into())
    .build(ctx)
    .transmute()
}

fn button(ctx: &mut BuildContext, label: Handle<UiNode>, width: f32, scale: f32) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
            .with_width(width * scale)
            .with_height(32.0 * scale)
            .with_uniform_margin(3.0 * scale),
    )
    .with_content(label)
    .build(ctx)
    .transmute()
}

/// N'envoie le texte à l'interface que s'il diffère de celui déjà affiché.
fn show(ui: &UserInterface, handle: Handle<UiNode>, shown: &mut String, value: String) {
    if *shown != value {
        ui.send(handle, TextMessage::Text(value.clone()));
        *shown = value;
    }
}

fn minutes(seconds: f64) -> String {
    format!("{:02}:{:02}", (seconds / 60.0) as u64, seconds as u64 % 60)
}

impl Hud {
    pub(crate) fn new(
        ui: &mut UserInterface,
        session: &Session,
        target: fyrox::resource::texture::TextureResource,
        scale: f32,
    ) -> Self {
        let ctx = &mut ui.build_ctx();
        let seed_value = session.simulation().seed();

        let brand = text(ctx, "CELL LAB", 28.0, ACCENT, scale);
        let subtitle = text(
            ctx,
            "ÉVOLUTION / ÉCOSYSTÈME EXPÉRIMENTAL",
            10.0,
            MUTED,
            scale,
        );
        let branding = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::left_right(14.0 * scale))
                .with_vertical_alignment(fyrox::gui::VerticalAlignment::Center)
                .with_children([brand, subtitle]),
        )
        .build(ctx);

        let cards = [
            ("CELLULES", INK),
            ("NAISSANCES", ACCENT),
            ("DÉCÈS", ROSE),
            ("ÉNERGIE MOY.", GOLD),
            ("TEMPS SIMULÉ", INK),
            ("SOUS L'EAU", SKY),
            ("PLUIE", SKY),
            ("SITES", ACCENT),
            ("CADENCE", MUTED),
        ]
        .map(|(label, color)| StatCard::new(ctx, label, "–", color, scale));
        let [cells, births, deaths, energy, time, water, rain, sites, pace] = cards;
        let stats = WrapPanelBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(4.0 * scale))
                .with_children([
                    cells.root(),
                    births.root(),
                    deaths.root(),
                    energy.root(),
                    time.root(),
                    water.root(),
                    rain.root(),
                    sites.root(),
                    pace.root(),
                ]),
        )
        .with_orientation(fyrox::gui::Orientation::Horizontal)
        .build(ctx);
        let header = GridBuilder::new(
            WidgetBuilder::new().with_children([branding.transmute(), stats.transmute()]),
        )
        .add_row(Row::auto())
        .add_column(Column::auto())
        .add_column(Column::stretch())
        .build(ctx);
        ctx[stats].set_column(1);
        let header = panel(
            ctx,
            header.transmute(),
            WidgetBuilder::new().on_row(0),
            Color::opaque(15, 25, 34),
            scale,
        );

        let mut buttons = Vec::new();
        let mut toolbar = Vec::new();
        let mut pause_text = Handle::NONE;
        let mut speed = Handle::NONE;
        for (label, width, action) in [
            ("Pause", 90.0, Some(Action::Pause)),
            ("Un pas", 70.0, Some(Action::Step)),
            ("−", 34.0, Some(Action::Slower)),
            ("× 1", 52.0, None),
            ("+", 34.0, Some(Action::Faster)),
            ("Vue", 80.0, Some(Action::CycleView)),
            ("Centrer", 80.0, Some(Action::Focus)),
        ] {
            let label_node = text(
                ctx,
                label,
                13.0,
                if action.is_some() { INK } else { ACCENT },
                scale,
            );
            match action {
                Some(action) => {
                    if action == Action::Pause {
                        pause_text = label_node;
                    }
                    let handle = button(ctx, label_node, width, scale);
                    buttons.push((handle, action));
                    toolbar.push(handle);
                }
                None => {
                    ctx[label_node].set_width(width * scale);
                    speed = label_node;
                    toolbar.push(label_node);
                }
            }
        }
        toolbar.push(text(ctx, "Graine", 12.0, MUTED, scale));
        let seed: Handle<UiNode> = TextBoxBuilder::new(
            WidgetBuilder::new()
                .with_width(100.0 * scale)
                .with_height(32.0 * scale)
                .with_uniform_margin(3.0 * scale),
        )
        .with_text(seed_value.to_string())
        .build(ctx)
        .transmute();
        toolbar.push(seed);
        for (label, width, action) in [
            ("Recommencer", 112.0, Action::Restart),
            ("Relire RON", 100.0, Action::Reload),
            ("Sauvegarder", 108.0, Action::Save),
            ("Charger", 88.0, Action::Load),
        ] {
            let label = text(ctx, label, 13.0, INK, scale);
            let handle = button(ctx, label, width, scale);
            buttons.push((handle, action));
            toolbar.push(handle);
        }
        let toolbar = WrapPanelBuilder::new(
            WidgetBuilder::new()
                .with_uniform_margin(6.0 * scale)
                .with_children(toolbar),
        )
        .with_orientation(fyrox::gui::Orientation::Horizontal)
        .build(ctx);
        let toolbar = panel(
            ctx,
            toolbar.transmute(),
            WidgetBuilder::new().on_row(1),
            Color::opaque(23, 35, 45),
            scale,
        );

        let heading = text(ctx, "ÉVOLUTION", 16.0, ACCENT, scale);
        let population_chart = SeriesChart::new(
            ctx,
            "Population, naissances et décès",
            &[("cellules", INK), ("naissances", ACCENT), ("décès", ROSE)],
            110.0,
            scale,
        );
        let energy_chart = SeriesChart::new(
            ctx,
            "Énergie moyenne par cellule",
            &[("énergie", GOLD)],
            80.0,
            scale,
        );
        let observatory = text(ctx, "OBSERVATOIRE", 16.0, ACCENT, scale);
        let inspector: Handle<UiNode> = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(4.0 * scale))
                .with_foreground(Brush::Solid(INK).into()),
        )
        .with_wrap(WrapMode::Word)
        .with_text("")
        .with_font_size((13.0 * scale).into())
        .build(ctx)
        .transmute();
        let content = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(10.0 * scale))
                .with_children([
                    heading,
                    population_chart.root(),
                    energy_chart.root(),
                    observatory,
                    inspector,
                ]),
        )
        .build(ctx);
        let scroll = ScrollViewerBuilder::new(WidgetBuilder::new())
            .with_content(content)
            .build(ctx);
        let sidebar = panel(
            ctx,
            scroll.transmute(),
            WidgetBuilder::new()
                .on_column(1)
                .with_margin(Thickness::uniform(8.0 * scale)),
            Color::opaque(19, 31, 41),
            scale,
        );
        let viewport = fyrox::gui::image::ImageBuilder::new(WidgetBuilder::new().on_column(0))
            .with_texture(target)
            .with_flip(true)
            .with_keep_aspect_ratio(false)
            .with_sync_with_texture_size(false)
            .build(ctx)
            .transmute();
        let body = GridBuilder::new(
            WidgetBuilder::new()
                .on_row(2)
                .with_children([viewport, sidebar]),
        )
        .add_row(Row::stretch())
        .add_column(Column::stretch())
        .add_column(Column::strict(360.0 * scale))
        .build(ctx);
        let status = text(
            ctx,
            "Prêt · clic gauche pour inspecter · clic droit pour orbiter · molette pour zoomer",
            12.0,
            MUTED,
            scale,
        );
        let footer = panel(
            ctx,
            status,
            WidgetBuilder::new().on_row(3),
            Color::opaque(15, 25, 34),
            scale,
        );
        let root = GridBuilder::new(
            WidgetBuilder::new()
                .with_name("Cell Lab HUD")
                .with_children([header, toolbar, body.transmute(), footer]),
        )
        .add_column(Column::stretch())
        .add_rows(vec![Row::auto(), Row::auto(), Row::stretch(), Row::auto()])
        .build(ctx)
        .transmute();
        Self {
            scale,
            frame: Vector2::new(1500.0, 950.0),
            root,
            body: body.transmute(),
            viewport,
            buttons,
            cells,
            births,
            deaths,
            energy,
            time,
            water,
            rain,
            sites,
            pace,
            population_chart,
            energy_chart,
            drawn: None,
            inspector,
            status,
            speed,
            pause_text,
            seed,
            editing_seed: false,
            seed_text: seed_value.to_string(),
            shown_inspector: String::new(),
            shown_speed: String::new(),
            shown_pause: None,
        }
    }

    pub(crate) fn set_viewport(
        &self,
        ui: &UserInterface,
        texture: fyrox::resource::texture::TextureResource,
    ) {
        ui.send(
            self.viewport,
            fyrox::gui::image::ImageMessage::Texture(Some(texture)),
        );
    }

    /// Zone d'affichage du monde 3D telle que calculée par la mise en page, en pixels
    /// de fenêtre : position du coin supérieur gauche et dimensions.
    pub(crate) fn viewport(&self, ui: &UserInterface) -> (Vector2<f32>, Vector2<f32>) {
        let bounds = ui.node(self.viewport).screen_bounds();
        (bounds.position, bounds.size)
    }

    fn sidebar_width(&self) -> f32 {
        (self.frame.x * 0.25).clamp(300.0 * self.scale, 480.0 * self.scale)
    }

    /// Largeur disponible aux tracés : barre latérale moins marges, bordures et ascenseur.
    fn plot_width(&self) -> f32 {
        (self.sidebar_width() - 72.0 * self.scale).max(60.0)
    }

    pub(crate) fn resize(&mut self, ui: &UserInterface, width: f32, height: f32) {
        self.frame = Vector2::new(width, height);
        ui.send(self.root, WidgetMessage::Width(width));
        ui.send(self.root, WidgetMessage::Height(height));
        ui.send(
            self.body,
            GridMessage::Columns(vec![
                Column::stretch(),
                Column::strict(self.sidebar_width()),
            ]),
        );
    }

    /// Repart d'une autre expérience : historique reconstruit depuis sa filiation et graine affichée.
    pub(crate) fn reset(&mut self, ui: &UserInterface, session: &Session) {
        self.drawn = None;
        self.shown_inspector.clear();
        self.seed_text = session.simulation().seed().to_string();
        ui.send(self.seed, TextMessage::Text(self.seed_text.clone()));
    }

    pub(crate) fn message(&mut self, message: &UiMessage) -> Option<Action> {
        if message.destination() == self.seed && message.direction() == MessageDirection::FromWidget
        {
            if let Some(event) = message.data::<WidgetMessage>() {
                match event {
                    WidgetMessage::Focus => self.editing_seed = true,
                    WidgetMessage::Unfocus => self.editing_seed = false,
                    _ => {}
                }
            }
            if let Some(TextMessage::Text(text)) = message.data() {
                self.seed_text = text.clone();
            }
        }
        if message.direction() == MessageDirection::FromWidget
            && matches!(message.data::<ButtonMessage>(), Some(ButtonMessage::Click))
        {
            self.buttons
                .iter()
                .find(|(handle, _)| *handle == message.destination())
                .map(|(_, action)| *action)
        } else {
            None
        }
    }

    pub(crate) fn status(&self, ui: &UserInterface, value: impl Into<String>) {
        ui.send(self.status, TextMessage::Text(value.into()));
    }

    /// Rafraîchit les statistiques, l'historique et l'inspecteur. `effective_speed` est la
    /// vitesse réellement obtenue, qui peut être inférieure à celle demandée si la machine
    /// ne suit pas.
    pub(crate) fn sync(&mut self, ui: &mut UserInterface, session: &Session, pacing: &Pacing) {
        let effective_speed = pacing.effective_speed();
        let sim = session.simulation();
        let stats = sim.summary();
        let requested = session.view().speed();

        let paused = session.view().paused();
        if self.shown_pause != Some(paused) {
            self.shown_pause = Some(paused);
            ui.send(
                self.pause_text,
                TextMessage::Text(if paused { "Reprendre" } else { "Pause" }.into()),
            );
        }
        show(
            ui,
            self.speed,
            &mut self.shown_speed,
            format!("× {requested}"),
        );

        self.cells.set(ui, stats.cells().to_string());
        self.births.set(ui, sim.generation().to_string());
        self.deaths.set(ui, stats.deaths().to_string());
        self.energy
            .set(ui, format!("{:.1}", stats.average_energy()));
        self.time.set(ui, minutes(stats.elapsed_seconds()));
        self.water
            .set(ui, format!("{:.1} %", stats.water_level() * 100.0));
        self.rain.set(
            ui,
            format!("{:.0} %", sim.grid().hydrology().summary().raining() * 100.0),
        );
        self.sites
            .set(ui, sim.grid().food_sites().len().to_string());
        self.pace.set(
            ui,
            if paused {
                "en pause".to_owned()
            } else if effective_speed < requested * 0.9 {
                format!(
                    "× {effective_speed:.1} (limité · retard {:.1} s)",
                    pacing.debt().as_secs_f32()
                )
            } else {
                format!("× {requested}")
            },
        );

        let history = session.history();
        let width = self.plot_width();
        let drawn = Some((history.revision(), width as u32));
        if self.drawn != drawn {
            self.drawn = drawn;
            self.population_chart.update(
                ui,
                width,
                history.samples(),
                &[
                    PopulationSample::cells,
                    PopulationSample::births,
                    PopulationSample::deaths,
                ],
            );
            self.energy_chart
                .update(ui, width, history.samples(), &[PopulationSample::energy]);
        }

        let detail = Self::inspection(session);
        show(ui, self.inspector, &mut self.shown_inspector, detail);
    }

    fn inspection(session: &Session) -> String {
        let sim = session.simulation();
        let mode = match session.view().mode() {
            ViewMode::Ecosystem => "Écosystème",
            ViewMode::Temperature => "Température",
            ViewMode::Adaptation => "Adaptation à l'eau",
            ViewMode::Strategy => "Stratégie énergétique",
        };
        let mut detail = format!(
            "{mode}\n{} · graine {}\n\n",
            if session.view().paused() {
                "EN PAUSE"
            } else {
                "EXPÉRIENCE EN COURS"
            },
            sim.seed()
        );
        if let Some(cell) = session.view().selected().and_then(|id| sim.cell(id)) {
            let (physiology, morphology) = (cell.physiology(), cell.morphology());
            let (genes, regulations, reach) = cell.genome().complexity();
            detail += &format!(
                "ESPÈCE\n{}\n{}\n\nORGANISME\n{}\n\nÉnergie     {:.1} / {:.1}\nÂge          {:.1} s\nDernière division  {:.1} s\nPosition     ({:.2}, {:.2})\nSite technique ({}, {})\nLiaisons          {}\n\nMORPHOLOGIE\nTaille {:.2} · réserve max {:.0}\nUnités nerveuses {}\nComposants : {}\n\nPHYSIOLOGIE\nMétabolisme  {:.3} / s (×{:.2} de taille)\nDivision à {:.0} d'énergie\nAdapt. eau {:.2} · terre {:.2}\nNage {:.3} site/s · mobilité {:.2}\nIngestion {:.2} · photosynthèse {:.2}\nThermosynthèse {:.2} · prédation {:.2}\nAdhésion {:.2} · réparation {:.2}\n\nGÉNOME\n{genes} gène(s) · {regulations} régulation(s) · {reach} effets\nÉvolution : chaque division mute de {:.2} fois en moyenne\n\nPRODUITS EXPRIMÉS (développement → maintenant)\n",
                sim.definitions().species().name(),
                cell.species_id(),
                cell.id(),
                cell.energy(),
                cell.capacity(),
                cell.age().as_secs_f32(),
                cell.last_division().as_secs_f32(),
                cell.position().0,
                cell.position().1,
                cell.x(),
                cell.y(),
                sim.bonds().degree(cell.id()),
                morphology.size(),
                morphology.capacity(),
                morphology.neural_units(),
                if morphology.anatomy().parts().is_empty() {
                    "aucun".to_owned()
                } else {
                    morphology
                        .anatomy()
                        .parts()
                        .iter()
                        .map(|part| format!("{} {}", part.count(), part.kind().label()))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                physiology.metabolism_rate(),
                physiology.upkeep_scale(),
                cell.division_threshold(),
                physiology.water_adaptation(),
                physiology.land_adaptation(),
                physiology.swim_speed(),
                physiology.motility(),
                physiology.ingestion(),
                physiology.photosynthesis(),
                physiology.thermosynthesis(),
                physiology.predation(),
                physiology.adhesion(),
                physiology.repair(),
                cell.genome().expected_mutations(physiology.repair(), 1.0),
            );
            for product in Product::ALL {
                detail += &format!(
                    "{:<17}{:.2} → {:.2}\n",
                    product.label(),
                    morphology.levels().get(product),
                    cell.levels().get(product)
                );
            }
            detail += "\nGÈNES (expression de base · réagit à · agit sur)\n";
            for (number, gene) in cell.genome().genes().iter().enumerate().take(12) {
                let reacts: Vec<String> = (0..REGULATORS)
                    .filter(|index| gene.input(*index) != 0.0)
                    .map(|index| format!("{} {:+.2}", regulator_label(index), gene.input(index)))
                    .collect();
                let acts: Vec<String> = Product::ALL
                    .iter()
                    .filter(|product| gene.effect(**product) != 0.0)
                    .map(|product| format!("{} {:+.2}", product.label(), gene.effect(*product)))
                    .collect();
                detail += &format!(
                    "g{} {:+.2} · {} · {}\n",
                    number + 1,
                    gene.bias(),
                    if reacts.is_empty() { "constitutif".to_owned() } else { reacts.join(", ") },
                    acts.join(", ")
                );
            }
            if cell.genome().genes().len() > 12 {
                detail += &format!("… {} autres gènes\n", cell.genome().genes().len() - 12);
            }
            detail += &format!(
                "\nNaissance : lumière {:.2} · chaleur {:.2} · eau {:.0} · nourriture {:.2}\n",
                cell.birth().get(Signal::Light),
                cell.birth().get(Signal::Warmth),
                cell.birth().get(Signal::Water),
                cell.birth().get(Signal::Food),
            );
            detail += &format!(
                "\nRÉSEAU NEURONAL\nCâblage {:.2} {:.2} {:.2}… biais {:.1}\nCapteur : {} · activation {:.3}\n",
                cell.genome().wiring()[0],
                cell.genome().wiring()[1],
                cell.genome().wiring()[2],
                cell.genome().neural_bias(),
                match cell.sensor() {
                    0 => "température",
                    1 => "lumière",
                    2 => "nourriture",
                    _ => "énergie",
                },
                cell.activation()
            );
            if let Some(record) = sim.lineage_record(cell.id()) {
                detail += &format!(
                    "\nFILIATION\nMère : {}\nNaissance : {:.2} s\n",
                    record
                        .parent()
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "fondatrice".into()),
                    record.born_at_tick() as f64 * FIXED_STEP.as_secs_f64()
                );
            }
        } else if let Some((id, (x, y))) = session.view().habitat().and_then(|id| {
            sim.grid()
                .habitat_position(id)
                .map(|position| (id, position))
        }) {
            let grid = sim.grid();
            let region = grid.region_at(x, y);
            let water = grid.hydrology();
            let medium = |environment: Environment| match environment {
                Environment::Water => "eau",
                Environment::Land => "terre",
            };
            let occupants = sim
                .cells()
                .iter()
                .filter(|c| c.x() == x && c.y() == y)
                .count();
            let kind = match grid.get_entity(x, y) {
                Entity::Empty => "Milieu libre",
                Entity::Food(_) => "Site alimentaire renouvelable",
                Entity::HeatSource(_) => "Source thermique",
                Entity::Obstacle(_) => "Substrat minéral",
                Entity::Water(_) => "Eau",
                Entity::Oxygen(_) => "Oxygène",
                Entity::Minerals(_) => "Ressource minérale",
            };
            detail += &format!(
                "SITE TECHNIQUE ({x}, {y})\n{id}\n\n{kind}\n{}\n\n{occupants} cellule(s)\nTempérature {:.1} °C (ambiante {:.1} °C)\nNourriture {:.2}\nMilieu : {}\n\nHABITAT RÉEL\n{} site(s) contigus · {} · {}\nTempérature moyenne {:.1} °C\n\nEAU ET RELIEF\nAltitude {:.2} · eau {:.2}\nCourant {:.2} site/s\nSol humide {:.2} · sédiments {:.3}\nHumidité de l'air {:.3} · nuage {:.3} · pluie {:.4}\nCharge électrique {:.3}\n\nLe site n'est qu'une maille de calcul :\nles organismes le traversent librement.",
                if grid.is_passable(x, y) {
                    "Traversable"
                } else {
                    "Infranchissable"
                },
                grid.get_temperature(x, y),
                grid.temperature_map().ambient_temperature(),
                grid.food_energy(x, y),
                medium(grid.get_environment_type(x, y)),
                region.sites(),
                medium(region.environment()),
                if region.passable() { "traversable" } else { "solide" },
                region.mean_temperature(),
                water.elevation(x, y),
                water.depth(x, y),
                water.flow_at(x, y).speed(),
                water.moisture(x, y),
                water.sediment(x, y),
                water.vapor(x, y),
                water.cloud(x, y),
                water.rain(x, y),
                grid.storm().charge(x, y, grid.size())
            );
        } else {
            let weather = sim.grid().hydrology().summary();
            let census = {
                let counts = sim.census(0.2);
                let mut lines: Vec<String> = [
                    Product::Digestive,
                    Product::Pigment,
                    Product::Thermal,
                    Product::Raptorial,
                    Product::Motor,
                    Product::Adhesin,
                    Product::Neural,
                ]
                .iter()
                .map(|product| format!("{} {}", product.label(), counts[product.index()]))
                .collect();
                let genes: usize = sim.cells().iter().map(|c| c.genome().genes().len()).sum();
                lines.push(format!(
                    "Gènes par organisme {:.1} · liaisons {}",
                    genes as f32 / sim.cells().len().max(1) as f32,
                    sim.bonds().len()
                ));
                lines.join("\n")
            };
            detail += &format!(
                "Cliquez une cellule pour observer\nson génome, son réseau et sa filiation.\n\nVIVANT\n{}\n\nMÉTÉO\nSous l'eau {:.0} % · pluie {:.0} %\nNuages {:.0} % · eau moyenne {:.2}\nCourant maximal {:.2} site/s\nEau totale {:.1}\nOrage : charge {:.2} · {} éclair(s) récents\n\nLe monde entier est colonisable.\nLes sites ne sont que des mailles\nde calcul partagées par leurs occupants.\n\nCAMÉRA\nClic droit : orbite\nMaj + clic droit : déplacement\nMolette : zoom\nF : centrer\n\nSIMULATION\nEspace : pause\nN : un pas\nCtrl+S : sauvegarder\nCtrl+O : charger\n\nCATALOGUE\nRévision {}\n\nAucune population n'est réintroduite\nautomatiquement en cas d'extinction.",
                census,
                weather.flooded() * 100.0,
                weather.raining() * 100.0,
                weather.cloud_cover() * 100.0,
                weather.mean_depth(),
                weather.max_speed(),
                weather.water(),
                sim.grid().storm().total(),
                sim.strikes().len(),
                sim.definitions().revision()
            );
        }
        detail
    }

    pub(crate) fn seed_text(&self) -> &str {
        &self.seed_text
    }
    pub(crate) fn editing_seed(&self) -> bool {
        self.editing_seed
    }
}
