# FerraSim

**Plateforme de simulation scientifique et industrielle, avec un moteur principal en Rust.**

FerraSim vise un environnement ouvert permettant de préparer, résoudre, vérifier, visualiser et automatiser des études mécaniques et multiphysiques. L’ambition est de devenir une alternative compétitive sur des parcours industriels identifiés face aux grandes suites de simulation, notamment SIMULIA.

Le projet commence par un solveur d’éléments finis structurel simple et vérifié, puis étend progressivement les formulations, les interfaces et les physiques. La précision, les bilans physiques et la reproductibilité guident le développement.

> **État actuel : résolution CG/Jacobi S05.** Le moteur Rust assemble et résout les systèmes TET4 réduits, avec déplacements imposés, réactions et diagnostics de convergence vérifiés sur des références. L’intégration des charges JSON et les exports restent à développer. La qualification scientifique indépendante reste ouverte.

## Pourquoi FerraSim ?

L’objectif est de rendre les études de simulation inspectables, automatisables et reproductibles :

- un modèle de données documenté et versionné ;
- un moteur indépendant de l’interface graphique ;
- des interfaces Rust, Python et CLI partageant les mêmes contrats ;
- des résultats accompagnés de leur provenance, des diagnostics et des bilans ;
- des formats d’échange ouverts et des intégrations de solveurs externes ;
- un fonctionnement local, puis sur cluster ou infrastructure choisie.

La compétitivité sera évaluée sur des parcours concrets, avec des preuves de précision, de robustesse, de productivité et de performance. La couverture annoncée restera liée aux domaines d’emploi effectivement validés.

## Compatibilité Windows, macOS et Linux

Les trois systèmes sont des cibles obligatoires dès le MVP. Le socle CPU, la CLI et la bibliothèque Rust, puis la GUI et l’API Python à leur livraison, doivent fonctionner nativement avec les mêmes fonctionnalités. Aucun WSL, Docker ou serveur distant n’est requis pour le socle local.

Le socle S04 et ses 73 tests de contrat, de géométrie, d’import, de physique élémentaire et d’assemblage passent la CI sur Windows, macOS et Linux. Les fonctionnalités physiques suivantes seront vérifiées sur les trois systèmes lors de leur implémentation. Chaque release indiquera les versions et architectures supportées ; les backends GPU/HPC et connecteurs spécialisés auront une matrice de disponibilité explicite.

## Premier objectif : MVP 0.1

La première version doit exécuter une analyse **statique d’élasticité linéaire en trois dimensions** depuis un fichier de modèle, sans interface graphique.

| Composant | Périmètre prévu |
| --- | --- |
| Modèle | JSON versionné, matériaux, groupes, prescriptions et charges |
| Maillage | Import Gmsh MSH 2.2 ASCII ; tétraèdres TET4 et triangles de frontière |
| Matériau | Élasticité linéaire isotrope |
| Conditions limites | Déplacements prescrits par composante, nuls ou non nuls |
| Chargement | Forces nodales et tractions surfaciques uniformes |
| Résolution | Assemblage creux en f64 ; CG préconditionné par Jacobi |
| Résultats | Déplacements, déformations, contraintes, von Mises et réactions |
| Exports | VTU, CSV et rapport JSON |
| Interfaces | Bibliothèque Rust et CLI |
| Validation | Tests élémentaires, patch test, équilibre, convergence et comparaison indépendante |

Le démonstrateur principal sera une poutre encastrée soumise à un chargement distribué. Les résultats seront vérifiés par une étude de convergence et des références documentées.

Le MVP exclut la CAO STEP, le maillage automatique, la GUI, Python, le contact, la plasticité, la dynamique, le GPU et le multiphysique. Ces capacités sont organisées dans les étapes suivantes.

## Vision du produit cible

Le produit cible réunira :

- **mécanique structurelle** : éléments enrichis, assemblages, grandes transformations, contact, dynamique implicite et explicite ;
- **matériaux et durabilité** : plasticité, hyperélasticité, composites, endommagement et fatigue ;
- **autres physiques** : thermique, CFD, électromagnétisme, acoustique et multicorps ;
- **multiphysique** : transferts de champs, couplages et co-simulation ;
- **environnement ingénieur** : CAO, maillage, GUI, visualisation et rapports ;
- **automatisation** : Python, campagnes, calibration, optimisation et gestion des travaux ;
- **calcul industriel** : parallélisme, HPC, reprise, traçabilité et maintenance.

Selon le domaine, les capacités pourront être natives ou fournies par des intégrations qualifiées. Chaque module devra publier ses hypothèses, ses limites et ses preuves de validation.

## Architecture prévue

```text
GUI / CLI / Python / API distante
                |
     Modèle versionné et commandes
                |
    Procédures et orchestration
                |
   Solveurs natifs / intégrations
                |
   Résultats, bilans et provenance
                |
     Visualisation et rapports
```

Le moteur principal est prévu en Rust. L’organisation initiale sera un Cargo workspace séparant modèle, maillage, FEM, solveurs, entrées/sorties et CLI. Les composants supplémentaires seront introduits avec leurs fonctionnalités.

## Roadmap

| Étape | Sprints de référence | Résultat attendu |
| --- | --- | --- |
| Fondations | S00–S11 | MVP 0.1 vérifié |
| Parcours structurel | S12–S23 | CAO/maillage, GUI, Python et première alternative ciblée |
| Assemblages non linéaires | S24–S35 | Matériaux non linéaires, contact et assemblages |
| Dynamique et durabilité | S36–S49 | Dynamique, fatigue, composites et qualification structurelle industrielle |
| Campagnes et optimisation | S50–S61 | Automatisation, optimisation et collaboration |
| Extension des physiques | S62–S79 | CFD, EM, acoustique et multicorps |
| Couplages et portefeuille | S80–S89 | Parcours multiphysiques et qualification du portefeuille |
| Approfondissement | S90–S99 | Modules avancés et audit de la vision cible |

La cadence proposée est de deux semaines. Les sprints lointains sont un découpage de référence à réestimer ; des cycles supplémentaires seront ajoutés lorsque les critères de validation l’exigent. Cette table ne constitue pas un calendrier de livraison garanti.

## Documentation

- [PRD du MVP 0.1](docs/PRD.md) — exigences et critères d’acceptation du premier moteur.
- [PRD de vision produit](docs/PRD-VISION.md) — cible industrielle, domaines et critères de compétitivité.
- [Roadmap générale](docs/ROADMAP.md) — organisation, dépendances et portes de qualification.
- [Sprints du MVP](docs/ROADMAP-MVP.md) — backlog de S00 à S11.
- [Sprints du produit cible](docs/ROADMAP-PRODUIT.md) — progression de S12 à S99.
- [Environnement de développement](docs/DEVELOPMENT.md) — installation et commandes de vérification.
- [Conventions numériques](docs/NUMERICAL-CONVENTIONS.md) — unités, tenseurs, TET4 et bilans.
- [Validation scientifique](validation/README.md) — cas, références et tolérances planifiées.
- [Rapport S00](docs/sprints/S00.md) — livrables, vérifications et points ouverts.
- [Contrat JSON 0.1](docs/MODEL-FORMAT.md) — format, règles de validation et limites.
- [Rapport S01](docs/sprints/S01.md) — modèle, IO et tests de contrat.
- [Contrat de maillage](docs/MESH-FORMAT.md) — sous-ensemble Gmsh, géométrie et groupes.
- [Rapport S02](docs/sprints/S02.md) — import, contrôles et limites.
- [Formulation TET4](docs/FEM-TET4.md) — matériau, opérateurs et références élémentaires.
- [Rapport S03](docs/sprints/S03.md) — physique locale et état de la revue.
- [Assemblage et prescriptions](docs/ASSEMBLY.md) — CSR, réduction, réactions et références.
- [Rapport S04](docs/sprints/S04.md) — livraison et vérifications globales.
- [Solveur CG/Jacobi](docs/SOLVER-CG.md) — convergence, erreurs et limites.
- [Rapport S05](docs/sprints/S05.md) — preuves de résolution.

## Développement

Les fondations **S00** comprennent le workspace, une version Rust fixée, les dépendances verrouillées, la licence MIT et les vérifications automatiques. Consulter le [guide de développement](docs/DEVELOPMENT.md) pour activer Rustup et installer les prérequis.

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p ferrasim-cli --locked -- --help
cargo run -p ferrasim-io --example inspect_model --locked -- examples/minimal/model.json
cargo run -p ferrasim-io --example inspect_mesh --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example inspect_element --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example inspect_assembly --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example inspect_solver --locked -- examples/minimal/model.json
```

La CLI affiche seulement l’aide et la version. `validate` et `solve` sont prévus en S08. `inspect_model` valide le JSON ; `inspect_mesh` vérifie maillage et références ; `inspect_element` évalue un champ artificiel sur un élément, sans résoudre les charges et contraintes du modèle. `inspect_assembly` construit la rigidité et sa réduction avec un vecteur de forces explicitement nul, sans intégrer les charges JSON ni résoudre le système. `inspect_solver` résout un scénario de force unitaire explicite ; il ne représente pas le chargement JSON.

## Contributions et licence

Les propositions peuvent être discutées dans les issues du dépôt. Voir les [règles de contribution](CONTRIBUTING.md) pour les vérifications, références scientifiques et revues.

FerraSim est distribué sous [licence MIT](LICENSE). Les licences des dépendances restent distinctes ; voir l’[inventaire des dépendances](docs/DEPENDENCIES.md) et la [décision de fondations](docs/decisions/0001-foundations.md).
