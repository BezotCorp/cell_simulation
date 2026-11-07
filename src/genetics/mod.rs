mod gene;
mod genome;
mod levels;
mod product;
mod signal;
mod signals;

#[cfg(test)]
mod tests;

pub(crate) use gene::{Gene, REGULATORS};
pub(crate) use genome::{Genome, MAX_GENES, NEURAL_UNITS, regulator_label};
pub(crate) use levels::Levels;
pub(crate) use product::Product;
pub(crate) use signal::Signal;
pub(crate) use signals::Signals;
