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
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-dev.txt
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_project.py
python scripts/check_model_schema.py
python3 validation/generate_tet4_reference.py --check
python3 validation/generate_patch_reference.py --check
python3 validation/generate_load_reference.py --check
cargo run -p ferrasim-cli --locked -- --help
cargo run -p ferrasim-io --example inspect_model --locked -- examples/minimal/model.json
cargo run -p ferrasim-io --example inspect_mesh --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example inspect_element --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example inspect_assembly --locked -- examples/minimal/model.json
cargo run -p ferrasim-solvers --example cg_reference --locked -- spd
python3 scripts/check_solver_failure.py
cargo run -p ferrasim-cli --example inspect_solver --locked -- examples/minimal/model.json
cargo run -p ferrasim-cli --example solve_model --locked -- examples/uniaxial/model.json 10 2
cargo run -p ferrasim-cli --example export_model --locked -- examples/uniaxial/model.json target/new-results 10 2
python3 scripts/check_result_exports.py target/new-results
python3 validation/verify_result_reference.py target/new-results
python3 scripts/check_export_failure.py
```

Les commandes de simulation `validate` et `solve` arriveront en S08. Le binaire actuel affiche seulement l’aide ou la version et refuse les autres commandes avec le code 2.

Sur Windows PowerShell, créer le venv avec `python -m venv .venv` et utiliser directement `.venv\Scripts\python.exe -m pip install -r requirements-dev.txt`, puis `.venv\Scripts\python.exe scripts/check_model_schema.py`. L’activation POSIX ci-dessus ne s’applique pas à PowerShell. Ce venv sert uniquement à vérifier le JSON Schema, pas à une API Python produit.

## Architecture initiale

| Package | Responsabilité |
| --- | --- |
| `ferrasim-model` | Contrats de modèle, identifiants, validation et erreurs |
| `ferrasim-mesh` | Connectivité, groupes et qualité |
| `ferrasim-solvers` | Stockage creux, résolution et diagnostics |
| `ferrasim-fem` | Matériaux, formulations, assemblage et résultats mécaniques |
| `ferrasim-io` | JSON, import maillage et exports |
| `ferrasim-cli` | Commandes et orchestration utilisateur |

Les frontières ont été créées en S00. S01 implémente le modèle typé et JSON ; S02 ajoute import MSH et contrôles du maillage/liaison ; S03 ajoute l’élasticité isotrope et les opérateurs TET4 locaux. S04 ajoute l’assemblage CSR, les prescriptions et les réactions ; S05 ajoute CG/Jacobi et ses rapports de convergence ; S06 intègre les charges et les bilans ; S07 ajoute champs immuables et exports ; les résolutions denses restent réservées aux tests. Voir les contrats [JSON](MODEL-FORMAT.md), [maillage](MESH-FORMAT.md), [élément](FEM-TET4.md), [assemblage](ASSEMBLY.md), [solveur](SOLVER-CG.md), [charges/bilans](LOADS-BALANCE.md) et [résultats](RESULTS-FORMAT.md).

## Plateformes et preuves

Windows, macOS et Linux sont des cibles obligatoires du socle. La CI configure les trois systèmes et s’exécute sur pushes à `main` et pull requests. Consulter le dernier run pour distinguer plateformes configurées et vérifiées. Le [rapport S00](sprints/S00.md) conserve les résultats locaux et distants effectivement observés.

Les nouvelles dépendances doivent être compatibles avec les trois OS avant adoption. Utiliser les API portables de chemins/processus ; ajouter les cas de fichiers Unicode, espaces et fins de ligne quand ces fonctions sont implémentées. Une différence de backend spécialisé doit être documentée et ne doit pas rendre l’installation CPU dépendante de WSL, Docker ou d’un service distant.

Les tests FEM de S03 vérifient les opérateurs locaux et leurs références denses. S04 vérifie un patch global multiélément et une référence hétérogène forcée avec résolution dense réservée aux tests. S05 vérifie CG contre LU, les références FEM et les scénarios d’échec explicites. S06 vérifie la traction uniaxiale et les bilans chargés. S07 vérifie les champs exportés et la lecture du VTU ; le contrôle VTK natif optionnel est décrit dans le [contrat](RESULTS-FORMAT.md). Les autres cas du [registre](../validation/cases.toml) restent planifiés ; la revue scientifique indépendante est ouverte.
