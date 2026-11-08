use crate::admin::Order;

/// Ce que la simulation apprend de la fenêtre d'administration.
#[derive(Debug)]
pub(crate) enum Incoming {
    /// Une fenêtre vient de se connecter.
    Connected,
    Order(Order),
}
