# Environnement de développement

## Prérequis

Rustup, toolchain Rust 1.96.0 avec Rustfmt/Clippy, Git et Python ≥ 3.11 pour les contrôles des documents et du registre de validation. Aucun BLAS natif, CMake, MPI, GPU ou environnement Python scientifique n’est requis pour S00.

Installer Rustup selon la [documentation officielle](https://rust-lang.org/tools/install/). Le fichier `rust-toolchain.toml` fixe la version et les composants du projet. Avec une installation Rustup standard, activer ses exécutables dans le terminal :

```sh
. "$HOME/.cargo/env"
rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
rustc --version
cargo --version
```

Sur Windows, utiliser l’installateur officiel et ses prérequis de compilation ; vérifier que les proxies Rustup sont dans PATH. La commande d’activation ci-dessus concerne les shells POSIX.

Sur le Mac de développement initial, Rust Homebrew reste installé. Rustup a été ajouté sans modifier les fichiers de configuration du shell ; l’activation ci-dessus est donc nécessaire dans chaque nouveau terminal si son PATH privilégie encore Homebrew.

## Cloner et vérifier

```sh
git clone https://github.com/ROSITO/ferrasim.git
cd ferrasim
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_project.py
cargo run -p ferrasim-cli --locked -- --help
```

Les commandes de simulation `validate` et `solve` arriveront en S08. Le binaire actuel affiche seulement l’aide ou la version et refuse les autres commandes avec le code 2.

## Architecture initiale

| Package | Responsabilité |
| --- | --- |
| `ferrasim-model` | Contrats de modèle, identifiants, validation et erreurs |
| `ferrasim-mesh` | Connectivité, groupes et qualité |
| `ferrasim-solvers` | Stockage creux, résolution et diagnostics |
| `ferrasim-fem` | Matériaux, formulations, assemblage et résultats mécaniques |
| `ferrasim-io` | JSON, import maillage et exports |
| `ferrasim-cli` | Commandes et orchestration utilisateur |

Les bibliothèques sont des frontières de package compilables, sans API physique ni modèle implémenté en S00. Les dépendances retenues sont compilées afin de vérifier leur compatibilité avec la toolchain. Leur usage métier commence aux sprints correspondants.

## Plateformes et preuves

La CI configure Linux, macOS et Windows et s’exécute sur pushes à `main` et pull requests. Consulter le dernier run pour distinguer plateformes configurées et vérifiées. Le [rapport S00](sprints/S00.md) conserve les résultats locaux et distants effectivement observés.

Compiler et réussir le test d’entrée CLI ne valide aucune formulation FEM. Les cas scientifiques du [registre](../validation/cases.toml) sont encore planifiés.
