mod admin;
mod application;
mod biology;
mod definitions;
mod ecosystem;
mod genetics;
mod habitat;
mod hydrology;
mod interface;
mod panel;
mod persistence;
mod world_scene;

#[cfg(test)]
mod test_support;

use application::{LaunchOptions, run};

fn main() {
    let result = LaunchOptions::parse().and_then(run);
    if let Err(error) = result {
        eprintln!("Cell Lab : {error}");
        std::process::exit(1);
    }
}
