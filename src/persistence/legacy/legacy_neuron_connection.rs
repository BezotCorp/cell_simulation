use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct LegacyNeuronConnection {
    pub(super) weight: f32,
    #[allow(dead_code)]
    from_id: usize,
    #[allow(dead_code)]
    to_id: usize,
}
