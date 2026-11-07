use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// Explicit algorithm and complete state, so a checkpoint resumes the next random draw.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SimulationRng(ChaCha12Rng);

impl SimulationRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(ChaCha12Rng::seed_from_u64(seed))
    }
}
impl rand::TryRng for SimulationRng {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.0.next_u32())
    }
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.next_u64())
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.0.fill_bytes(dst);
        Ok(())
    }
}
