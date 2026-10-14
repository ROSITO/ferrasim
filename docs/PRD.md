# FerraSim — Document de besoins produit

Version : 0.1 — proposition de cadrage\
Date : 14 octobre 2026\
Statut : base de discussion avant implémentation

Ce document spécifie le MVP 0.1. La destination à long terme, les domaines de simulation et les critères de compétitivité industrielle sont définis dans le [PRD de vision produit](PRD-VISION.md).

Le plan d’exécution est détaillé dans la [roadmap générale](ROADMAP.md) et les [sprints S00 à S11 du MVP](ROADMAP-MVP.md).

## 1. Résumé et décision de périmètre

FerraSim est un projet de plateforme open source de simulation scientifique, avec un moteur de calcul en Rust, un modèle de données documenté et des interfaces permettant l’automatisation.

La première version, FerraSim 0.1, répond à un besoin précis : **décrire, résoudre et vérifier une analyse statique d’élasticité linéaire en trois dimensions sur un maillage tétraédrique**. Elle fonctionne sans interface graphique et exporte ses résultats pour leur inspection dans un outil externe.

La vision multiphysique guide la séparation des composants. Elle ne constitue pas une exigence de livraison du MVP. La prise en charge de la CAO, du maillage automatique, de Python et d’un environnement graphique est organisée en incréments distincts.

Le positionnement initial est celui d’un moteur FEM ouvert, reproductible et scriptable. Toute comparaison de couverture ou de précision avec une suite commerciale devra être appuyée par des fonctionnalités livrées et des validations publiées.

## 2. Problème utilisateur

Un ingénieur ou chercheur doit pouvoir rejouer un calcul, comprendre ses hypothèses, vérifier son équilibre mécanique et récupérer des résultats exploitables sans dépendre d’une interface graphique.

FerraSim doit rendre explicites :

- les unités, hypothèses et limites de l’analyse ;
- les régions auxquelles s’appliquent matériaux, charges et conditions limites ;
- les erreurs de définition du modèle et les échecs numériques ;
- la provenance des résultats et les critères de validation.

## 3. Utilisateurs et usages prioritaires

| Utilisateur | Besoin prioritaire | Réponse de la version 0.1 |
| --- | --- | --- |
| Développeur de méthodes numériques | Comprendre et vérifier les formulations | API Rust, exemples et tests de référence |
| Chercheur | Rejouer et comparer des calculs | Fichiers de modèle, CLI et résultats exportables |
| Ingénieur évaluant le projet | Évaluer un cas mécanique simple | Démonstrateur de poutre, déplacements, contraintes et réactions |
| Enseignant | Illustrer une analyse FEM et ses limites | Petits modèles documentés et exercices de convergence |

Les études de certification, les grandes assemblées industrielles et les problèmes fortement non linéaires ne font pas partie des usages garantis de la version 0.1.

## 4. Objectifs et exclusions

### Objectifs de FerraSim 0.1

1. Exécuter un calcul statique linéaire 3D complet depuis un fichier de modèle.
2. Produire déplacements, déformations, contraintes, contraintes de von Mises et réactions.
3. Détecter les entrées invalides et signaler les problèmes de convergence ou de stabilité.
4. Publier des validations analytiques et un protocole de comparaison externe reproductible.
5. Fournir une API Rust utilisable indépendamment de la CLI.

### Exclusions de la version 0.1

- Import STEP/IGES et noyau de géométrie B-Rep.
- Génération automatique de maillages à partir d’une CAO.
- Éléments HEX8, poutres, coques et formulations 2D.
- Contact, plasticité, grandes déformations et dynamique.
- Thermique, CFD, électromagnétisme et couplage multiphysique.
- Interface graphique, API REST et service de calcul distant.
- API Python stable, solveurs externes exécutés comme plugins.
- MPI, GPU et engagements de performance HPC.

Ces fonctionnalités restent des perspectives. Leur présence dans la roadmap ne vaut pas engagement de livraison.

## 5. Parcours de référence

### Parcours obligatoire en 0.1

1. L’utilisateur prépare un maillage et ses groupes nommés.
2. Il écrit un fichier décrivant matériau, contraintes cinématiques, charges et paramètres de résolution.
3. Il valide le modèle avec la CLI.
4. Il lance la résolution dans un répertoire de sortie.
5. Il consulte le rapport, puis ouvre le fichier VTU dans un visualiseur compatible.
6. Il rejoue le calcul avec un maillage plus fin et compare les résultats.

Commandes cibles :

```sh
ferrasim validate examples/cantilever/model.json
ferrasim solve examples/cantilever/model.json --output results/cantilever
```

### Parcours ultérieur

Importer une géométrie STEP, générer un maillage, sélectionner les faces, définir l’analyse, résoudre et afficher les résultats dans FerraSim. Ce parcours nécessite des composants de géométrie, de maillage et d’interface supplémentaires ; il sera spécifié après validation du moteur.

## 6. Hypothèses physiques de la version 0.1

- Mécanique des solides en trois dimensions.
- Analyse statique, petites déformations et petits déplacements.
- Matériau élastique linéaire isotrope, défini par le module d’Young et le coefficient de Poisson.
- Un matériau peut être attribué à chaque région volumique ; les affectations sont exhaustives et sans ambiguïté.
- Trois degrés de liberté de translation par nœud.
- Éléments tétraédriques linéaires à quatre nœuds, TET4.
- Aucune stabilisation destinée à traiter l’incompressibilité n’est garantie.

Le modèle utilise un système cohérent d’unités déclaré par l’utilisateur. Les exemples et validations sont en SI. La version 0.1 n’effectue pas de conversion automatique : les longueurs, modules et forces doivent être cohérents. Le rapport rappelle cette convention.

Le TET4 est choisi pour limiter la première formulation à vérifier. Sa convergence en flexion et son comportement pour des matériaux presque incompressibles doivent être documentés. Un résultat calculé ne suffit pas à établir la pertinence du maillage.

## 7. Exigences fonctionnelles

Les exigences suivantes sont obligatoires pour déclarer la version 0.1 livrée.

| ID | Exigence | Critère d’acceptation |
| --- | --- | --- |
| MOD-01 | Charger un modèle JSON versionné | Schéma documenté, exemple minimal et rejet explicite des versions inconnues |
| MOD-02 | Référencer un maillage et des groupes nommés | Références absentes, groupes vides utilisés et identifiants dupliqués signalés |
| MSH-01 | Importer Gmsh MSH 2.2 ASCII | Nœuds, TET4 et triangles de frontière importés ; groupes physiques conservés ; formats ou éléments non pris en charge rejetés |
| MSH-02 | Vérifier le maillage | Connectivité, coordonnées finies, volumes, orientations et surfaces sélectionnées contrôlés avant assemblage |
| MAT-01 | Définir l’élasticité isotrope | Module strictement positif et coefficient de Poisson dans l’intervalle physique ouvert −1 < ν < 0,5 ; proximité de l’incompressibilité signalée |
| BC-01 | Imposer des déplacements par composante sur un groupe de nœuds | Valeurs nulles ou non nulles prises en charge ; prescriptions contradictoires rejetées |
| LOAD-01 | Appliquer des forces nodales | Chaque valeur correspond à la force appliquée à un nœud identifié ; aucune duplication implicite d’une résultante |
| LOAD-02 | Appliquer une traction vectorielle uniforme à une surface | Charge intégrée sur les triangles de frontière ; résultante conforme à la traction et à l’aire |
| FEM-01 | Assembler et résoudre Ku = F en précision f64 | Tests élémentaires, patch test et contrôles de résidu passent |
| SOL-01 | Résoudre avec CG préconditionné par Jacobi | Tolérance et nombre maximal d’itérations configurables ; état de convergence explicite |
| SOL-02 | Signaler un système invalide ou non convergent | Erreur exploitable ; aucune sortie présentée comme un succès après échec |
| RES-01 | Calculer les champs et réactions | Déplacements nodaux ; déformations et contraintes par élément ; réactions sur les degrés de liberté prescrits |
| IO-01 | Exporter VTU, CSV et rapport JSON | Champs, localisation, ordre des composantes et unités documentés ; VTU lisible dans un outil compatible |
| CLI-01 | Valider et résoudre sans GUI | Codes de sortie distincts pour succès, entrée invalide et échec de résolution |
| API-01 | Exposer le moteur comme bibliothèque Rust | Exemple Rust reproduisant le calcul de la CLI sans dépendance à l’interface utilisateur |

L’importeur Gmsh doit documenter la construction des groupes de nœuds à partir des régions et des surfaces. Une sélection de surface ne doit pas reposer sur une recherche approximative par coordonnées.

Les pressions normales, forces volumiques et déplacements pilotés dans le temps sont hors périmètre initial. Ils pourront être ajoutés après spécification de leurs conventions.

## 8. Modèle de données et contrats

Le modèle sépare les données physiques, les paramètres numériques et les demandes de sortie :

```text
Simulation
├── schema_version
├── units
├── mesh
├── materials
├── material_assignments
├── boundary_conditions
├── loads
├── analysis
├── solver_settings
└── outputs
```

Principes obligatoires :

- Les entités possèdent des identifiants stables et les sélections utilisent des groupes nommés.
- Une configuration inconnue ou ambiguë déclenche une erreur ; aucun défaut silencieux ne modifie la physique.
- L’ordre des composantes des tenseurs et la convention des déformations de cisaillement sont documentés et partagés par le matériau, l’assemblage et l’export.
- Les contraintes TET4 sont constantes par élément. Tout lissage nodal futur devra être distingué du champ calculé.
- Le calcul des réactions utilise le système original, avant modification pour imposer les déplacements.
- Le rapport identifie le modèle, le maillage, la version du moteur, les paramètres du solveur et les fichiers de résultat.

Géométrie, interactions et étapes multiples seront ajoutées lorsque leurs besoins seront établis. Le schéma ne doit pas promettre des fonctionnalités encore absentes.

## 9. Architecture attendue

Le moteur doit rester indépendant des interfaces utilisateur. La première organisation est un Cargo workspace limité aux responsabilités nécessaires :

```text
crates/
  model/       # données, identifiants, validations et erreurs
  mesh/        # connectivité, régions et contrôles de maillage
  fem/         # matériaux, TET4, assemblage et post-traitement mécanique
  solvers/     # matrices creuses et résolution
  io/          # modèle JSON, import MSH, exports
apps/
  ferrasim-cli/
examples/
validation/
docs/
```

Les bibliothèques sont utilisables sans la CLI. Les composants de géométrie, scripting et GUI seront créés quand ils porteront une fonctionnalité réelle.

La matrice globale utilise un stockage creux. Un solveur dense de référence peut servir aux petits tests, mais ne constitue pas le chemin de production. La politique de dépendances devra examiner licence, maintenance, portabilité et adéquation numérique avant choix des bibliothèques.

Les interfaces seront extensibles là où plusieurs implémentations sont nécessaires. Une ABI de plugins dynamiques, un graphe de couplage et des abstractions GPU ne sont pas requis en 0.1.

## 10. Robustesse et exigences non fonctionnelles

- Aucune panique pour une erreur utilisateur connue : le moteur retourne une erreur structurée.
- Valeurs non finies, éléments dégénérés, affectations incomplètes et prescriptions incompatibles sont contrôlés.
- Les contrôles simples de sous-contrainte sont complétés par les diagnostics du solveur. La détection de toute singularité ne doit pas être annoncée sans preuve.
- La résolution conserve un statut, le nombre d’itérations et un résidu relatif recalculé sur le système réduit.
- Le critère de convergence définit séparément le cas d’un second membre nul, à l’aide d’une tolérance absolue documentée.
- Un calcul identique doit reproduire ses résultats dans les tolérances publiées. L’identité bit à bit entre architectures n’est pas exigée.
- Les fichiers de résultats ne sont déclarés complets qu’après succès ; les diagnostics d’échec sont conservés séparément.
- La CLI est conçue pour Linux, macOS et Windows ; les plateformes effectivement testées sont indiquées dans chaque livraison.
- Temps d’assemblage, temps de résolution, nombre de degrés de liberté, nombre de coefficients et estimation mémoire sont rapportés.

Les objectifs de taille et de vitesse seront fixés à partir de mesures sur une machine de référence documentée. Aucun seuil de performance n’est une promesse avant l’existence d’un benchmark reproductible.

## 11. Validation scientifique et critères de sortie

Les validations doivent vérifier la formulation, l’assemblage, les charges, les conditions limites et les résultats. Chaque cas conserve ses entrées, sa référence, sa métrique, sa tolérance et les instructions de reproduction.

### Suite obligatoire

| Cas | Ce qui est vérifié | Cible d’acceptation initiale |
| --- | --- | --- |
| TET4 isolé | Symétrie de la rigidité et modes rigides | Erreurs normalisées ≤ 10⁻¹⁰ sur élément bien conditionné |
| Patch test avec déplacement affine imposé | Reproduction d’une déformation constante | Erreurs normalisées de déplacement et contrainte ≤ 10⁻⁸ |
| Barre prismatique en traction uniforme | Déplacement axial et contrainte | Erreur relative ≤ 10⁻⁶ sur le cas construit pour une solution affine |
| Déplacements prescrits non nuls | Réduction du système et reconstruction | Accord avec une référence dense sur un petit modèle, erreur relative ≤ 10⁻⁸ |
| Traction surfacique uniforme | Intégration de la charge | Erreur relative de résultante ≤ 10⁻¹⁰ sur une surface simple |
| Équilibre global | Somme des charges et réactions, bilan des moments | Déséquilibre normalisé ≤ 10⁻⁶, avec échelles de force et de longueur explicites |
| Poutre encastrée en flexion | Convergence en déplacement | Au moins trois niveaux de raffinement ; cible d’écart ≤ 5 % sur le maillage final retenu, selon le protocole ci-dessous |
| Modèles invalides | Qualité des diagnostics | Rejet des cas dégénérés, références absentes et contraintes contradictoires ; échec explicite sur cas singulier de référence |

Les tolérances ci-dessus sont des exigences proposées. Toute révision doit être justifiée par le conditionnement ou la référence utilisée, puis documentée avant validation de la livraison.

### Protocole de poutre

La formule d’Euler–Bernoulli δ = FL³/(3EI) sert de référence approchée pour une poutre suffisamment élancée. Le protocole fixe dimensions, chargement distribué, conditions d’encastrement et définition du déplacement mesuré. Il distingue erreur de discrétisation et différence entre modèles de poutre et de solide 3D.

Le seuil de 5 % porte sur le déplacement défini par ce protocole, jamais sur une contrainte locale au voisinage d’une singularité. Si les hypothèses de poutre ne permettent pas une comparaison pertinente, la référence est remplacée par un calcul solide 3D indépendant et convergé, avec justification publiée.

### Comparaison externe

Avant livraison 0.1, au moins un cas doit être comparé à un solveur indépendant, par exemple CalculiX, sur le même maillage, avec le même matériau, les mêmes charges et les mêmes prescriptions. Les fichiers et instructions sont publiés ; l’exécution de ce logiciel n’est pas requise pour utiliser FerraSim. Les écarts de déplacement et de contrainte par élément doivent être ≤ 10⁻⁵ en métrique relative documentée sur un cas bien conditionné, hors valeurs nulles traitées par une échelle absolue.

La plaque trouée vient ensuite comme démonstrateur de convergence des contraintes. Une valeur au bord du trou ne constitue pas un critère valide sans définir géométrie, référence, position d’évaluation et raffinement.

## 12. Livrables et définition de terminé

FerraSim 0.1 est livrée lorsque :

1. Toutes les exigences obligatoires de la section 7 sont satisfaites.
2. La suite de validation obligatoire passe et la comparaison externe est documentée.
3. Un utilisateur peut reproduire le démonstrateur depuis une installation propre en suivant le README.
4. Le modèle d’entrée, les sorties, les unités, les hypothèses et les limites numériques sont documentés.
5. Le rapport distingue clairement succès et échec, avec les diagnostics nécessaires.
6. Une licence open source a été choisie et sa compatibilité avec les dépendances vérifiée.
7. Les plateformes testées et les résultats du benchmark initial sont publiés.

Livrables : code Rust, CLI, API Rust, modèles d’exemple, maillages de validation, exports de référence, documentation et résultats des vérifications automatisées.

## 13. Plan de développement par jalons

| Jalon | Résultat utilisable | Condition de passage |
| --- | --- | --- |
| J0 — Contrats | Schéma de modèle, conventions, choix de licence et dépendances | Exemples d’entrée et protocole de validation définis |
| J1 — Élément et matériau | TET4 et élasticité isotrope | Tests de rigidité, modes rigides et patch test passent |
| J2 — Assemblage et résolution | Petit modèle résolu via API Rust | Traction affine, prescriptions non nulles et équilibre passent |
| J3 — Chaîne utilisateur | Import MSH, CLI et exports | Calcul complet depuis un modèle JSON, diagnostic et VTU exploitables |
| J4 — Livraison 0.1 | Démonstrateur reproductible et validation publiée | Définition de terminé satisfaite |

L’estimation de charge et le calendrier seront établis après J0, selon l’équipe, les bibliothèques retenues et le temps nécessaire à la validation numérique.

## 14. Roadmap après 0.1

La roadmap décrit un ordre d’exploration. Chaque incrément aura son propre périmètre et ses critères de sortie.

| Incrément | Valeur attendue | Dépendance ou condition |
| --- | --- | --- |
| API Python | Automatiser les modèles et exploiter les résultats dans l’écosystème scientifique | Contrats Rust et formats stabilisés ; stratégie de packaging définie |
| Maillage depuis une CAO | Réaliser le parcours STEP → maillage → calcul | Évaluation d’un noyau ou outil externe, licences et qualité de maillage |
| Autres éléments structurels | Améliorer les usages en flexion et les modèles 2D | Validation séparée par formulation, intégration et modes parasites |
| Interface graphique | Construire et inspecter les modèles | API de modèle et résultats suffisamment stables |
| Thermique stationnaire | Ajouter une deuxième physique | Bilans et validations thermiques indépendants |
| Non-linéaire et dynamique | Étendre les phénomènes mécaniques | Gestion d’état, convergence, pas de temps et vérification d’énergie |
| Orchestration externe | Réutiliser des solveurs existants | Contrats d’échange, processus, erreurs et provenance définis |
| Couplages multiphysiques | Enchaîner ou coupler plusieurs physiques | Mapping conservatif, unités, convergence et validation des couplages |
| Parallélisme avancé | Traiter des modèles plus grands | Profils de calcul et goulots d’étranglement mesurés |

CFD, électromagnétisme, optimisation, MPI et GPU restent des axes de recherche à long terme. Ils ne déterminent pas la date de livraison de 0.1.

## 15. Risques et décisions à prendre

| Risque | Conséquence | Réponse prévue |
| --- | --- | --- |
| Périmètre trop large | Moteur non validé malgré de nombreuses interfaces | Un seul type d’analyse et un seul élément dans le MVP |
| Raideur excessive des TET4 en flexion | Déplacement sous-estimé | Étude de convergence publiée et limites explicites |
| Matériau presque incompressible | Mauvaise précision ou conditionnement | Avertissement et exclusion des performances garanties dans ce régime |
| Sous-contrainte ou système mal conditionné | Résolution impossible ou résultats trompeurs | Validation du modèle, diagnostics et cas d’échec dédiés |
| Ambiguïté sur les charges | Résultante incorrecte | Distinction explicite entre force nodale et traction surfacique |
| Dépendances incompatibles avec la distribution | Blocage de publication | Choix de licence et audit des dépendances au jalon J0 |
| Généralisation prématurée | Complexité sans bénéfice utilisateur | Extension guidée par les incréments réellement livrés |

Décisions ouvertes avant implémentation : licence, bibliothèque de matrices creuses, stratégie de distribution et machine de benchmark. Les choix de cadrage proposés ici sont JSON, MSH 2.2 ASCII, TET4, CG/Jacobi et VTU/CSV ; ils constituent le point de départ du développement, sous réserve de validation au jalon J0.
