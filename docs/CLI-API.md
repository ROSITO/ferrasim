# CLI et API Rust de bout en bout — S08

Le binaire natif `ferrasim` valide et résout les modèles JSON 0.1 sur Windows, macOS et Linux. Aucun Python, WSL, Docker ou serveur n’est nécessaire au calcul et aux exports. Python sert uniquement aux contrôles indépendants de développement.

## Utilisation

Depuis le dépôt, avec la toolchain Rust fixée :

```sh
cargo build --workspace --release --locked
cargo run -p ferrasim-cli --locked -- validate examples/uniaxial/model.json
cargo run -p ferrasim-cli --locked -- solve examples/uniaxial/model.json --output target/new-cli-results
```

Le binaire release est `target/release/ferrasim` (`ferrasim.exe` sur Windows). Les mêmes arguments s’appliquent directement :

```sh
ferrasim validate model.json
ferrasim solve model.json --output new-results
ferrasim solve model.json --output other-results --force-scale 10 --length-scale 2 --origin 0 0 0
```

`--help`/`-h`, `--version`/`-V` et l’aide de chaque sous-commande sont disponibles. Le répertoire parent de sortie doit exister et la destination doit être absente ; changer son nom avant une nouvelle exécution. Aucun `--force` ou écrasement implicite.

## Validation et résolution

`validate` charge le JSON, importe le maillage et contrôle géométrie, groupes, affectations, prescriptions, matériaux et intégration des charges. Les opérateurs FEM sont préparés pour vérifier leur représentabilité. Aucun CG n’est exécuté et aucun fichier de résultat n’est écrit. Une validation réussie ne prouve pas l’absence de singularité, la convergence ou la qualification scientifique.

`solve` prépare les mêmes entrées, résout les charges réelles, récupère les champs et exporte les formats demandés. Les tolérances et le budget CG viennent du JSON ; la CLI ne les modifie pas. Les avertissements matériau sont visibles sur stderr. Le succès est affiché sur stdout seulement après publication complète ; le manifeste final S07 reste le critère de complétude du répertoire.

Le statut numérique et les valeurs de bilan restent distincts : `Converged` atteste le critère de résidu CG ; les bilans sont publiés pour évaluation avec leurs échelles. La CLI ne transforme pas cette convergence en qualification scientifique indépendante.

## Codes de sortie et erreurs

| Code | Signification |
| --- | --- |
| 0 | Aide/version, validation réussie ou calcul publié complètement |
| 2 | Usage incorrect ou options invalides |
| 3 | Entrée, maillage, liaison ou préparation physique/numérique invalide |
| 4 | Échec solveur ou récupération des champs |
| 5 | Destination ou écriture des résultats impossible |

Les erreurs vont sur stderr, sans message de succès sur stdout. Les erreurs sémantiques donnent les chemins de champs ; les diagnostics de géométrie/liaison conservent les IDs d’entités lorsqu’ils existent. Une limite CG donne notamment `IterationBudget` et le résidu recalculé. Les options inconnues, répétées, incomplètes et valeurs non finies sont rejetées ; les échelles doivent être positives.

L’ordre des étapes est explicite : options, chargement/liaison, précontrôle non destructif de destination, préparation, résolution/récupération, publication. Une destination existante peut donc être signalée avant un problème numérique ultérieur. Les erreurs de préparation sont de code 3, même quand le modèle était syntaxiquement valide.

## Chemins et sorties

Les arguments de fichiers restent des chemins natifs via `OsString`/`PathBuf`. Espaces et Unicode sont couverts sur les trois OS. Les chemins de maillage du JSON restent relatifs au modèle, indépendamment du répertoire de travail. Le chemin `--output` relatif est, lui, résolu depuis le répertoire de travail de l’appelant.

Les options peuvent précéder le chemin du modèle. Un modèle dont le nom commence par un tiret utilise le séparateur `--`, après les options :

```sh
ferrasim solve --output new-results -- -model.json
```

Les fichiers Unix à noms non UTF-8 sont testés sur Linux, dont le système de fichiers accepte ces noms ; la provenance textuelle les représente avec remplacement si nécessaire. Ce n’est pas une promesse que tous les systèmes de fichiers acceptent ces octets.

Les destinations existantes sont rejetées, y compris fichiers, répertoires vides/non vides et liens symboliques. Le précontrôle ne réserve pas la destination ; la création exclusive S07 reste le garde-fou lors de l’écriture. Aucune création automatique de parents. Les limites d’écriture partielle/nettoyage et de concurrence sont celles du [contrat des résultats](RESULTS-FORMAT.md).

## Échelles des bilans

Les options `--force-scale` et `--length-scale` modifient uniquement les dénominateurs des bilans, jamais les charges, les unités ou le critère CG. `--origin X Y Z` fixe l’origine des moments, par défaut `(0,0,0)`. Les valeurs effectivement utilisées sont enregistrées dans le rapport.

Sans options explicites :

- Force : maximum entre la somme des normes des forces nodales intégrées et `max(abs(Kij))*max(abs(u_prescrit))`. Si ces deux valeurs sont nulles, repli explicite à 1 unité cohérente de force.
- Longueur : diagonale de la boîte englobante du maillage.

Ce sont des échelles de normalisation documentées, pas des critères physiques universels. La somme des normes évite la division par une résultante qui s’annule ; le terme de prescriptions couvre les chargements par déplacement. Les calculs hors plage sont rejetés ; des échelles explicites peuvent être fournies. Les études comparatives doivent fixer les mêmes échelles. Pour le bloc uniaxial, les valeurs automatiques sont environ 10 N et √6 m ; les preuves S06 utilisaient explicitement 10 N et 2 m.

## API indépendante de la CLI

`ferrasim-io` expose :

- `validate_simulation_file(path) -> Result<ValidationSummary, SimulationRunError>`.
- `run_simulation_file(path, output, &RunOptions) -> Result<SimulationRun, SimulationRunError>`.

`RunOptions` contient les deux échelles optionnelles et l’origine. `SimulationRun` contient le résultat immuable et la liste des fichiers publiés. Les erreurs sont typées par étape et proposent `exit_code()` et `diagnostic()` ; un appelant Rust peut traiter les variantes directement. Ni IO ni FEM ne dépendent du package CLI.

L’[exemple Rust](../crates/io/examples/run_simulation.rs) utilise la même API sans lancer de processus CLI :

```sh
cargo run -p ferrasim-io --example run_simulation --locked -- examples/uniaxial/model.json target/new-api-results
```

Cette première orchestration prépare l’assemblage pour les échelles puis le reconstruit dans le chemin FEM de récupération, en libérant le premier avant résolution. Cette surcharge est explicite et n’est pas une promesse de performance ; elle préserve les contrats immuables existants. Aucun solveur dense n’est introduit.

## Preuves et reproduction

Les tests natifs CLI couvrent aide, usage, chemins Unicode/espaces, répertoire de travail distinct, options avant chemin, nom commençant par un tiret, valeurs numériques, diagnostics, codes de sortie, protection des données, cas sans charge et comparaison des exports CLI/API. Les tests API couvrent échelles automatiques, options invalides et erreur matérielle avant solveur. Linux ajoute le cas de chemin non UTF-8.

Le [script de reproduction](../scripts/check_cli_workflow.py) copie le modèle et son maillage dans un répertoire temporaire Unicode, choisit un autre répertoire de travail, lance la CLI et l’exemple API natifs et compare tous leurs fichiers. Les lecteurs croisés S07 contrôlent ensuite les champs et le manifeste. La CI répète la construction depuis un checkout propre sur les trois systèmes ; les prérequis de compilation restent ceux du [guide de développement](DEVELOPMENT.md).

```sh
cargo test --workspace --locked
cargo build -p ferrasim-cli --locked
cargo build -p ferrasim-io --example run_simulation --locked
python3 scripts/check_cli_workflow.py
```

La poutre, la convergence de maillage, la comparaison externe et la qualification indépendante ne sont pas acquises par ce parcours de bout en bout. S09 prépare le démonstrateur et l’étude de convergence ; la revue scientifique reste ouverte.
