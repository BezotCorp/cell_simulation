#[derive(Clone)]
pub(crate) struct NeuronConnection {
    weight: f32,
    from_id: usize,
    to_id: usize,
}

impl NeuronConnection {
    pub(crate) fn new(weight: f32, from_id: usize, to_id: usize) -> Self {
        Self {
            weight,
            from_id,
            to_id,
        }
    }

    pub(crate) fn weight(&self) -> f32 {
        self.weight
    }

    pub(crate) fn from_id(&self) -> usize {
        self.from_id
    }

    pub(crate) fn to_id(&self) -> usize {
        self.to_id
    }

    pub(crate) fn weight_mut(&mut self) -> &mut f32 {
        &mut self.weight
    }
}
