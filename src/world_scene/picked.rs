use uuid::Uuid;
#[derive(Clone, Copy, Debug)]
pub(crate) enum Picked {
    Organism(Uuid),
    Habitat(Uuid),
}
