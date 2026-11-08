use super::Runtime;
use crate::admin::{AdminLink, Incoming, Order, Report, Status};
use crate::application::LaunchOptions;
use crate::definitions::Definitions;
use crate::genetics::Product;
use crate::persistence::Session;
use fyrox::plugin::PluginContext;
use std::{sync::Arc, time::Duration};

/// Période d'envoi de l'état de l'expérience à la fenêtre d'administration.
const STATUS_PERIOD: Duration = Duration::from_millis(500);

impl Runtime {
    /// Traite ce que la fenêtre d'administration a demandé et lui envoie l'état de temps en temps.
    pub(crate) fn serve(&mut self, link: &AdminLink, options: &LaunchOptions, context: &mut PluginContext) {
        for incoming in link.poll() {
            match incoming {
                Incoming::Connected => link.send(&self.definitions_report(options)),
                Incoming::Order(order) => {
                    let report = match self.obey(order, options, context) {
                        Ok(Some(report)) => report,
                        Ok(None) => Report::Done { ok: true, message: "Fait".into() },
                        Err(error) => Report::Done { ok: false, message: error },
                    };
                    link.send(&report);
                }
            }
        }
        if link.connected() && self.since_status.elapsed() >= STATUS_PERIOD {
            self.since_status = std::time::Instant::now();
            link.send(&Report::Status(self.status()));
        }
    }

    fn definitions_report(&self, options: &LaunchOptions) -> Report {
        let definitions = self.session.simulation().definitions();
        Report::Definitions {
            ron: ron::ser::to_string(&**definitions).unwrap_or_default(),
            save_path: options.save_path().display().to_string(),
        }
    }

    fn status(&self) -> Status {
        let simulation = self.session.simulation();
        let summary = simulation.summary();
        let genomes: std::collections::HashSet<u64> =
            simulation.cells().iter().map(|cell| cell.genome().id()).collect();
        let census = simulation.census(0.25);
        Status {
            seconds: summary.elapsed_seconds(),
            cells: summary.cells(),
            births: summary.births(),
            deaths: summary.deaths(),
            average_energy: summary.average_energy(),
            flooded: summary.water_level(),
            paused: self.session.view().paused(),
            speed: self.session.view().speed(),
            world_size: simulation.grid().size(),
            genomes: genomes.len(),
            census: Product::ALL.iter().map(|p| (p.label().to_owned(), census[p.index()])).collect(),
        }
    }

    /// Exécute une demande ; l'expérience n'est jamais laissée à moitié modifiée.
    fn obey(
        &mut self,
        order: Order,
        options: &LaunchOptions,
        context: &mut PluginContext,
    ) -> Result<Option<Report>, String> {
        let message = match order {
            Order::Intervene(intervention) => self.session.intervene(&intervention)?,
            Order::Redefine(text) => {
                let definitions = Self::parse(&text)?;
                self.session.redefine(definitions)?;
                self.rebuild_scene(context);
                "Définitions appliquées à l'expérience en cours".into()
            }
            Order::Restart { definitions, seed } => {
                let definitions = Self::parse(&definitions)?;
                let seed = seed.unwrap_or(definitions.default_seed());
                let session = Session::new(definitions, seed)?;
                self.session.save(options.save_path().with_extension("previous.bin"))?;
                self.install(session, true, context);
                "Nouvelle expérience (la précédente est dans previous.bin)".into()
            }
            Order::SetPaused(paused) => {
                self.session.set_paused(paused);
                if paused { "En pause" } else { "Reprise" }.into()
            }
            Order::Step => {
                self.session.step_once();
                "Un pas".into()
            }
            Order::SetSpeed(speed) => {
                self.session.set_speed(speed);
                format!("Vitesse × {speed}")
            }
            Order::Save => {
                self.session.save(options.save_path())?;
                format!("Expérience sauvegardée · {}", options.save_path().display())
            }
            Order::Refresh => return Ok(Some(self.definitions_report(options))),
        };
        Ok(Some(Report::Done { ok: true, message }))
    }

    fn parse(text: &str) -> Result<Arc<Definitions>, String> {
        let definitions: Definitions = ron::from_str(text).map_err(|e| format!("Définitions illisibles : {e}"))?;
        definitions.validate()?;
        Ok(Arc::new(definitions))
    }
}
