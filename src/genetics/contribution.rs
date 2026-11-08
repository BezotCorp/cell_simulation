use crate::genetics::{Condition, Product};

/// Contribution compilée : ajoute `amount` (multiplié par le nombre de copies du motif de dose,
/// borné) à un produit quand toutes ses conditions sont vraies.
#[derive(Clone, Debug)]
pub(crate) struct Contribution {
    product: Product,
    amount: f32,
    conditions: Vec<Condition>,
    per_copy: Option<(usize, u32)>,
}

impl Contribution {
    pub(crate) fn new(
        product: Product,
        amount: f32,
        conditions: Vec<Condition>,
        per_copy: Option<(usize, u32)>,
    ) -> Self {
        Self {
            product,
            amount,
            conditions,
            per_copy,
        }
    }

    pub(crate) fn product(&self) -> Product {
        self.product
    }

    pub(crate) fn amount(&self) -> f32 {
        self.amount
    }

    pub(crate) fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    pub(crate) fn per_copy(&self) -> Option<(usize, u32)> {
        self.per_copy
    }

    /// Vrai si une condition dépend du milieu : la contribution est alors réévaluée à chaque expression.
    pub(crate) fn is_dynamic(&self) -> bool {
        self.conditions.iter().any(Condition::is_dynamic)
    }
}
