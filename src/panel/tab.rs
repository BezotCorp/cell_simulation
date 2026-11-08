/// Onglets de la fenêtre d'administration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tab {
    Live,
    Capabilities,
    Appearance,
    Start,
    Configs,
}

impl Tab {
    pub(crate) const ALL: [Tab; 5] = [Self::Live, Self::Capabilities, Self::Appearance, Self::Start, Self::Configs];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Live => "🎮 Pilotage",
            Self::Capabilities => "🧬 Capacités",
            Self::Appearance => "🎨 Rendus 3D",
            Self::Start => "🌱 Départ",
            Self::Configs => "💾 Configurations",
        }
    }
}
