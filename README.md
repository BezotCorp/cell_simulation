# Cell Lab — Fyrox 3D

Laboratoire d’évolution cellulaire en Rust. La scène, les matériaux PBR, les lumières,
la caméra scriptée et l’interface sont natifs Fyrox. La biologie conserve une grille
torique de micro-habitats : la représentation est 3D, les déplacements biologiques
restent cardinaux. La sélection est un lancer de rayon analytique (plan du biotope et
sphères des organismes) : aucun corps ni collision physique n’existe dans la scène,
et rien n’impose d’exclusion entre cellules.

## Lancer

Depuis ce dossier, avec Rust et les bibliothèques système nécessaires à Fyrox :

```sh
cargo run --locked
cargo run --locked -- --seed 7
cargo run --locked -- --load runtime/session.bin
cargo run --locked -- --headless --seed 7 --steps 450000 --save runtime/experiment.bin
```

Le mode sans fenêtre exécute exactement le même modèle biologique et écrit un vrai
checkpoint BIN. `--load` restaure une expérience et `--steps` indique les pas
supplémentaires à exécuter en mode sans fenêtre.

## Manipuler l’expérience

- Clic gauche : sélectionner un organisme ou un habitat et lire l’inspecteur.
- Clic droit maintenu : orbiter ; Maj + clic droit : déplacer la cible ; molette : zoomer.
- Espace : pause/reprise ; N : avancer de 16 ms ; F : centrer la sélection.
- Boutons − / + : vitesse de 0,25 à 64 ; Vue : écosystème, température, adaptation à l’eau.
  La pastille « Cadence » indique la vitesse réellement obtenue ; si la machine ne suit pas,
  elle affiche « limité » et le retard accumulé (voir *Temps et charge*).
- Graine puis Recommencer : nouvelle expérience reproductible.
- Relire RON : valider les définitions éditées et commencer une nouvelle expérience.
- Ctrl+S / Sauvegarder : checkpoint ; Ctrl+O / Charger : restaurer ce checkpoint.

Avant un redémarrage ou un rechargement RON, l’expérience courante est conservée
dans `runtime/session.previous.bin`. À la fermeture normale, une sauvegarde distincte
`runtime/session.autosave.bin` conserve le dernier état sans écraser le checkpoint manuel.
Les erreurs sont affichées dans le pied de fenêtre ; une opération échouée ne remplace
pas la scène courante. Les chemins dépendent de `--save` (voir `--help`).

## Temps et charge

La biologie avance exclusivement par pas de 16 ms. À haute vitesse, la simulation reçoit à chaque
image un budget de calcul (≈ 9 ms) ; le nombre de pas exécutés s’adapte au coût mesuré. Le temps
simulé demandé mais non exécuté n’est pas perdu : il reste dû, dans la limite de quatre fois la
demande d’une image, et la machine ralentit l’expérience au lieu de s’endetter indéfiniment
(le ralentissement est affiché, jamais silencieux). Le temps simulé est compté en pas : il ne
dépend ni de la cadence d’affichage ni de la charge, et un même état avance de façon identique
quelle que soit la machine. Le retard est un état d’exécution, il n’est pas sauvegardé.
Une suspension de la fenêtre de plus de 250 ms n’est pas rattrapée.

## Données et architecture

`data/definitions.ron` est l’unique catalogue humain : espèce, génome initial,
réseau ancestral, budgets énergétiques, environnement et matériaux. Les placements
explicites permettent de définir murs, minéraux, nourriture ou sources thermiques
sans modifier les règles biologiques. Les minéraux sont des substrats observables ;
ils ne procurent pas d’énergie, comme dans le modèle biologique existant.

Les définitions sont partagées par `Arc`, les références persistantes utilisent
UUIDv7. Les cellules ne dupliquent pas les paramètres de leur espèce. Une sauvegarde
contient le modèle runtime, la filiation, l’état complet du générateur ChaCha,
l’accumulateur temporel, la sélection, la caméra, la pause et la vitesse. La scène
Fyrox et ses caches graphiques sont reconstruits à partir de cet état.

Les fichiers BIN utilisent `bincode-next` avec un en-tête versionné et une empreinte
de la partie biologique du catalogue (identité, espèce et monde). L’apparence — titre,
matériaux, rayon et espacement de rendu — n’en fait pas partie : la retoucher n’invalide
aucune sauvegarde. La version 2 écrit cette empreinte ; la version 1, dont l’empreinte
couvrait tout le catalogue, reste lisible et son contenu est identique. Le chargement refuse un catalogue modifié, une version inconnue,
un fichier tronqué, les données supplémentaires et les états incohérents.
L’écriture passe par un fichier temporaire synchronisé puis un renommage atomique.
Les index de filiation, de sites alimentaires et d’identités d’habitats sont dérivés : ils
sont reconstruits à la restauration. Toute opération de chargement, de redémarrage ou de
relecture prépare et valide la nouvelle expérience (et sa scène) avant de remplacer l’ancienne ;
une erreur laisse l’expérience courante intacte et l’écriture de `previous.bin` est atomique.
Le catalogue n’est pas recopié dans chaque checkpoint. L’empreinte détecte les
changements ; elle n’est pas une signature cryptographique.

Les responsabilités se répartissent entre `biology`, `habitat`, `ecosystem`,
`definitions`, `persistence`, `world_scene`, `interface` et `application`.
Les `mod.rs` ne contiennent que déclarations et exports. Les tests unitaires sont
dans les sous-modules `tests` des domaines ; les tests d’intégration sont dans
`tests/` et utilisent uniquement l’interface publique du programme : la CLI et les fichiers BIN.
Le projet reste un exécutable, sans `lib.rs`. Les types et opérations partagés sont
limités à `pub(crate)` ; tous les champs restent privés et les mutations passent
par les méthodes des propriétaires de l’état.

## Interface et rendu

- La mise en page suit la fenêtre : barre d’outils et pastilles de statistiques à retour à la
  ligne, colonne latérale proportionnelle (300 à 480 px), échelle de l’interface selon la densité
  de pixels de l’écran. La zone 3D, le picking et la cible de rendu utilisent la zone réellement
  calculée par l’interface ; la cible n’est recréée qu’après stabilisation du redimensionnement.
- Les courbes de la colonne latérale montrent effectifs, naissances cumulées, décès cumulés et énergie
  moyenne sur toute la durée de l’expérience. L’historique se condense (un point sur deux) au lieu de
  s’interrompre. Il n’est pas sauvegardé : après un chargement, effectifs, naissances et décès sont
  reconstruits depuis la filiation complète ; l’énergie moyenne repart de la restauration.
- La scène n’est resynchronisée que lorsque la simulation a avancé ou que la vue change, et seuls les
  éléments dont l’apparence diffère touchent au graphe. Les habitats vides n’ont pas d’objet de
  ressource ; la nourriture se met à jour par paliers de hauteur. Les matériaux (dégradés de 64
  paliers) et textures 1×1 sont partagés ; les sources thermiques éclairent à raison de 12 lumières
  au plus (les autres restent émissives) et les lumières ponctuelles ne projettent pas d’ombre.
- Les déplacements, croissances et naissances sont lissés en temps réel : indépendants de la
  cadence d’affichage, de la vitesse et du pas biologique. Les habitats très peuplés disposent leurs
  occupants en spirale de Vogel dans la dalle, avec un rayon réduit pour éviter tout recouvrement :
  aucun individu n’est masqué.

## Vérifier

```sh
cargo check --locked --offline --all-targets
cargo test --locked --offline
```

Le scénario prolongé vérifie 16 graines pendant deux heures simulées chacune,
sans extinction ni dépassement de l’enveloppe de régression, ainsi que la poursuite
des naissances et des mutations à l’échelle de l’ensemble. La survie universelle
n’est pas garantie. Voir [le cycle de vie](docs/life-cycle.md) et
[le réseau neuronal et l’environnement](docs/neural-environment.md).

Le contrôle graphique s’effectue depuis l’extérieur de l’application : lancer le
binaire, manipuler les commandes natives, inspecter le rendu, puis vérifier la
reprise du fichier sauvegardé. Aucun scénario de test, injection de messages UI
ou rapport de test n’est embarqué dans le code de production.
