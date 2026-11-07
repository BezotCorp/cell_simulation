//! Coefficients du cycle de l'eau. Ils décrivent des propriétés physiques du monde (en unités de
//! hauteur d'eau par site et par seconde simulée), pas des événements : aucune pluie, crue ou
//! sécheresse n'est programmée, elles naissent de ces échanges et du relief.

/// Profondeur à partir de laquelle un site est submergé (milieu aquatique pour les organismes).
pub(crate) const WET_DEPTH: f32 = 0.05;
/// Épaisseur d'eau qui enveloppe entièrement un organisme : en deçà, le courant ne le saisit que
/// partiellement.
pub(crate) const BODY_DEPTH: f32 = 0.15;
/// Vitesse maximale du courant, en sites par seconde.
pub(crate) const MAX_SPEED: f32 = 2.0;

/// Vapeur saturante à 20 °C au niveau moyen du relief, et température d'e-folding.
pub(super) const SATURATION_REFERENCE: f32 = 0.2;
pub(super) const SATURATION_SCALE: f32 = 12.0;
/// Refroidissement par unité d'altitude : la montagne condense, la plaine évapore.
pub(super) const LAPSE_RATE: f32 = 5.0;
/// Fractions du déficit de saturation reprises par seconde.
pub(super) const EVAPORATION: f32 = 0.05;
pub(super) const SOIL_EVAPORATION: f32 = 0.3;
pub(super) const CONDENSATION: f32 = 0.5;
pub(super) const CLOUD_EVAPORATION: f32 = 0.3;
/// Diffusion de l'air vers chacun des quatre voisins.
pub(super) const AIR_DIFFUSION: f32 = 0.03;
/// Seuil de nuage au-delà duquel les gouttes grossissent et tombent, et part tombant par seconde.
pub(super) const CLOUD_CRITICAL: f32 = 0.06;
pub(super) const RAIN_FRACTION: f32 = 0.25;
/// Infiltration : débit maximal par unité de perméabilité et de porosité libre.
pub(super) const INFILTRATION: f32 = 0.04;
/// Part de l'excès de saturation du sol qui ressort par seconde.
pub(super) const EXFILTRATION: f32 = 0.5;
/// Écoulement : fraction de la dénivelée de surface transférée par face et par seconde (quatre
/// faces × 0,2 < 1 : le schéma ne fait qu'aplanir, il n'oscille pas).
pub(super) const FLOW_COEFFICIENT: f32 = 0.2;
/// Épaisseur de référence qui borne la vitesse d'une lame d'eau très mince.
pub(super) const SPEED_DEPTH: f32 = 0.08;
/// Capacité de transport (∝ vitesse²), taux d'arrachement et de dépôt, et variation maximale
/// d'altitude par seconde.
pub(super) const TRANSPORT_CAPACITY: f32 = 0.02;
pub(super) const EROSION_RATE: f32 = 0.05;
pub(super) const DEPOSITION_RATE: f32 = 0.2;
pub(super) const MAX_RELIEF_CHANGE: f32 = 0.01;
/// Profondeur sous laquelle l'écoulement est négligé et le sédiment se dépose.
pub(super) const DRY_DEPTH: f32 = 1e-4;
/// Surélévation des sites solides (roches, murs) : l'eau les contourne.
pub(super) const SOLID_RAISE: f32 = 0.7;
/// Itérations d'écoulement qui installent lacs et bassins avant le premier pas.
pub(super) const SETTLING_TICKS: usize = 60;
/// Part des sites non solides initialement sous l'eau.
pub(crate) const INITIAL_FILL: f32 = 0.85;
/// Amplitude du relief initial, en hauteurs d'eau : la pente règle la vitesse des écoulements.
pub(super) const RELIEF: f32 = 2.0;

/// Durée du jour, en secondes simulées, et opacité de la couverture nuageuse à la lumière.
pub(super) const DAY_SECONDS: f32 = 150.0;
pub(super) const CLOUD_OPACITY: f32 = 8.0;
/// Atténuation de la lumière par mètre d'eau limpide, et supplément par unité de turbidité.
pub(super) const WATER_CLARITY: f32 = 1.2;
pub(super) const WATER_MUD: f32 = 3.0;

/// Électrification : charge produite par unité de pluie et de nuage, fuite par seconde (plus forte
/// dans l'air clair), seuil de claquage, probabilité maximale de décharge par seconde et part de la
/// charge libérée par un éclair.
pub(super) const CHARGE_RATE: f32 = 35.0;
pub(super) const CHARGE_LEAK: f32 = 0.02;
pub(super) const CLEAR_AIR_LEAK: f32 = 0.35;
pub(super) const BREAKDOWN: f32 = 0.5;
pub(super) const STRIKE_PROBABILITY: f32 = 0.4;
pub(super) const STRIKE_RELEASE: f32 = 0.85;
/// Eau évaporée par unité d'énergie d'éclair.
pub(super) const FLASH_BOIL: f32 = 0.25;
/// Densité de nuage sous laquelle l'air est considéré clair pour la fuite des charges.
pub(super) const CLOUD_CRITICAL_FOR_LEAK: f32 = 0.01;
