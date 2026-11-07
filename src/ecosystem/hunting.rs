/// Index des occupants de chaque site pour la chasse : `members[heads[s]..heads[s + 1]]` sont les
/// organismes du site `s`. Les tampons sont réutilisés d'un pas à l'autre.
#[derive(Clone, Default)]
pub(crate) struct Hunting {
    pub(super) heads: Vec<u32>,
    pub(super) members: Vec<u32>,
    pub(super) fill: Vec<u32>,
}
