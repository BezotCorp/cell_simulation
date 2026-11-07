use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct LaunchOptions {
    definitions: PathBuf,
    save_path: PathBuf,
    load_path: Option<PathBuf>,
    seed: Option<u64>,
    headless: bool,
    steps: u64,
    help: bool,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            definitions: PathBuf::from("data/definitions.ron"),
            save_path: PathBuf::from("runtime/session.bin"),
            load_path: None,
            seed: None,
            headless: false,
            steps: 3750,
            help: false,
        }
    }
}

impl LaunchOptions {
    pub(crate) fn parse() -> Result<Self, String> {
        Self::from_arguments(std::env::args().skip(1))
    }

    fn from_arguments(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut result = Self::default();
        let mut args = args.into_iter();

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--definitions" => {
                    result.definitions = args.next().ok_or("Chemin RON manquant")?.into();
                }
                "--save" => {
                    result.save_path = args.next().ok_or("Chemin BIN manquant")?.into();
                }
                "--load" => {
                    result.load_path = Some(args.next().ok_or("Chemin BIN manquant")?.into());
                }
                "--seed" => {
                    result.seed = Some(
                        args.next()
                            .ok_or("Graine manquante")?
                            .parse()
                            .map_err(|_| "Graine entière invalide")?,
                    );
                }
                "--steps" => {
                    result.steps = args
                        .next()
                        .ok_or("Nombre de pas manquant")?
                        .parse()
                        .map_err(|_| "Nombre de pas invalide")?;
                }
                "--headless" => {
                    result.headless = true;
                }
                "--help" | "-h" => {
                    result.help = true;
                }
                other => {
                    return Err(format!("Argument inconnu : {other}. Utilisez --help."));
                }
            }
        }

        Ok(result)
    }

    pub(crate) fn definitions(&self) -> &Path {
        &self.definitions
    }

    pub(crate) fn save_path(&self) -> &Path {
        &self.save_path
    }

    pub(crate) fn load_path(&self) -> Option<&Path> {
        self.load_path.as_deref()
    }

    pub(crate) fn seed(&self) -> Option<u64> {
        self.seed
    }

    pub(crate) fn headless(&self) -> bool {
        self.headless
    }

    pub(crate) fn steps(&self) -> u64 {
        self.steps
    }

    pub(crate) fn help(&self) -> bool {
        self.help
    }
}
