# Contribuer à FerraSim

Le projet suit la [roadmap](docs/ROADMAP.md). Les changements doivent rester dans le sprint retenu ; le modèle et le solveur ne sont pas implémentés au sprint S00.

## Vérifications avant contribution

Avec la toolchain du projet et Python 3.11 ou supérieur :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_project.py
python scripts/check_model_schema.py
python3 validation/generate_tet4_reference.py --check
```

`Cargo.lock` est versionné pour toute la plateforme. Toute mise à jour de dépendance inclut une justification, ses licences et les régressions pertinentes. Les crates ne sont pas publiées sur crates.io à ce stade.

Le contrôle indépendant du JSON Schema utilise un venv avec `requirements-dev.txt` ; voir le [guide de développement](docs/DEVELOPMENT.md). Les dépendances Python de vérification sont distinctes du moteur Rust.

## Règles scientifiques

- Définir référence, métrique et tolérance avant calculer le résultat.
- Documenter unités, repères, ordre des composantes et domaine d’emploi.
- Tester le comportement physique et les modes d’échec pertinents.
- Conserver les références et données nécessaires à la reproduction ; ne pas importer de données sans droits de redistribution.
- Faire relire une nouvelle formulation par une personne compétente distincte de son auteur avant qualification.
- Ne pas présenter un test de build comme une validation scientifique.

## Règles de code et de revue

Séparer modèle, maillage, FEM, solveurs et IO ; garder la CLI sans physique. Retourner des erreurs pour les entrées utilisateur ; documenter les API publiques. Aucun `unsafe` dans le code du workspace à ce stade ; cela ne garantit pas son absence dans les dépendances.

Une contribution précise le problème, le périmètre, les vérifications effectuées et les limites. Ne pas ajouter de données privées, secrets ou résultats volumineux. Les fichiers de projet inconnus ne doivent pas exécuter automatiquement du code.

Les contributions au dépôt sont distribuées sous sa licence MIT. Les licences des dépendances et données tierces restent distinctes.
