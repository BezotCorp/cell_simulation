# Cycle de l'eau, relief et courants

Le module `hydrology` gouverne l'eau du monde.
Il est déterministe (aucun tirage aléatoire :
le vent est une fonction de la graine et du temps simulé), avance d'une seconde simulée à la fois, et ne dépend que des différences de hauteur, de température et de saturation entre sites voisins.
Aucune pluie, crue ou sécheresse n'est programmée : elles naissent des échanges ci-dessous.

## Grandeurs (par site, en hauteur d'eau équivalente)

| Champ           | Rôle                                                                               |
| --------------- | ---------------------------------------------------------------------------------- |
| altitude        | relief du sol, modelé par l'érosion et le dépôt                                    |
| profondeur      | eau libre en surface ; un site est submergé à partir de 0,05                       |
| humidité du sol | eau retenue, bornée par la porosité (sable 0,20, sol nu 0,28, terre 0,35, roche 0) |
| sédiments       | matière arrachée, portée par l'eau                                                 |
| vapeur, nuage   | eau de l'air, transportée par le vent                                              |
| courant         | débit de surface, en sites par seconde (≤ 2)                                       |

## Un pas d'une seconde

1. **Évaporation** : proportionnelle au déficit de saturation de l'air.
   La vapeur saturante croît exponentiellement avec la température (e-folding 12 °C) et diminue avec l'altitude (5 °C par unité) :
   les sources de chaleur évaporent beaucoup, les reliefs condensent.
2. **Vent** : la vapeur et les nuages sont transportés (schéma amont conservatif) dans une direction qui dérive lentement, avec une légère diffusion.
   Un nuage porté par le vent précipite loin de la mer qui l'a formé : la pluie est locale, l'ombre pluviométrique émerge du relief.
3. **Condensation** quand l'air dépasse la saturation, **évaporation des nuages** sinon.
4. **Précipitation** : au-delà d'un seuil de nuage, la pluie croît plus vite que la densité du nuage (les gros nuages précipitent bien plus que les voiles minces).
5. **Infiltration** : l'eau de surface s'enfonce selon la perméabilité du sol (sable 1, sol nu 0,5, terre 0,35, roche 0) et la place libre ; l'excès de saturation ressort en surface.
6. **Écoulement** : chaque site cède à ses quatre voisins plus bas une fraction (0,2) de la dénivelée de surface libre (`altitude + profondeur`), sans jamais céder plus qu'il n'a.
   Le schéma ne fait qu'aplanir : il ne crée pas d'oscillation. Le courant du site est le débit traversant divisé par `profondeur + 0,08`, borné à 2 sites/s.
7. **Érosion et dépôt** : la capacité de transport croît avec le carré de la vitesse.
   En dessous, le courant arrache de la matière selon l'érodabilité du sol (sable 1, sol nu 0,7, terre 0,5, roche 0) ; au-dessus, il dépose.
   Les sédiments suivent l'eau (concentration conservée). Un site asséché dépose tout. Le relief modifié redirige aussitôt l'eau du pas suivant.
8. **Perte géologique** : `water_decay_per_second` du catalogue reste la fraction d'eau de surface et du sol conservée par seconde (0,9999 : le monde s'assèche lentement, comme avant).

Le monde commence avec 85 % de ses sites non solides sous l'eau (l'ancêtre est aquatique et apparaît dans l'eau si le monde en contient) ;
soixante écoulements installent d'abord lacs et bassins. Les sites `Water` du catalogue ajoutent une flaque initiale.
Les roches et les murs sont surélevés, imperméables et inérodables.

## Effets sur le monde et les organismes

- **Milieu** : un site submergé est un milieu aquatique pour l'entretien des cellules
  (`water_adaptation`), un site émergé un milieu terrestre (`land_adaptation`). Inondation et
  sécheresse changent donc réellement les coûts de survie, localement.
- **Courant** : un organisme nage contre l'écoulement avec toute sa poussée disponible
  (`0,12 site/s × adaptation à l'eau × métabolisme`), ce qui lui coûte jusqu'à 30 % de son
  métabolisme. L'excédent l'emporte : un organisme sans adaptation aquatique est entraîné par le
  moindre courant, un organisme adapté résiste aux courants modérés et peut les remonter. Une
  lame d'eau plus mince qu'un corps (0,15) ne l'entraîne que partiellement. Le déplacement ne
  traverse jamais un solide. L'eau peut ainsi coloniser des régions lointaines, ou déposer des
  cellules dans des conditions qu'elles ne supportent pas : le sort de la cellule découle ensuite
  des règles biologiques inchangées.
- **Sauvegarde** : version 5, quatrième section (relief, eau, sédiments, air, courants). Les
  versions antérieures reconstruisent l'hydrologie d'après leur niveau d'eau global.

## Rendu

Une surcouche translucide non éclairée (shader dédié, passe _Forward_) suit les textures peintes
depuis l'état hydrologique, interpolées entre les centres de sites : relief ombré (ravines et
dépôts visibles), sol détrempé ou desséché, nappes dont la couleur va du turquoise des bas-fonds au
bleu des bassins et se charge de boue avec les sédiments. Un second plan porte les nuages. La pluie
(640 gouttes réutilisées) tombe uniquement sur les sites où il pleut, et des traînées claires
dérivent le long des courants. La vue thermique les masque. Les textures ne sont repeintes qu'une
fois le cycle avancé, au plus quatre fois par seconde.

## Orages et lumière

Le cycle de l'eau alimente l'électrification de l'atmosphère et la lumière que reçoivent les
organismes : voir [evolution.md](evolution.md). Les nuages voilent la lumière (opacité 8 par unité de
nuage, jusqu'à 85 %), l'eau l'atténue (1,2 par unité de profondeur, jusqu'à 4,2 avec la boue) et le
jour dure 150 s. La pluie et les nuages chargent l'atmosphère ; un éclair vaporise l'eau du site
touché (elle retourne dans l'air).
