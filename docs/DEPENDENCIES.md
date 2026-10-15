# Inventaire des dépendances

Date : 15 octobre 2026. Source : `cargo metadata --format-version 1 --locked`, graphe des features par défaut.

S01 ajoute l’usage direct de Serde et thiserror dans IO, sans nouvelle version de package Rust.

S02 ajoute l’usage direct de thiserror dans mesh, sans nouveau package externe ni nouvelle version.

S03 ajoute thiserror dans FEM et Serde/serde_json comme dépendances de test, sans nouveau package externe ni nouvelle version. Le générateur rationnel utilise seulement la bibliothèque standard Python.

S04 utilise le CSR de nalgebra-sparse déjà présent et ajoute thiserror comme dépendance directe de solvers. Aucun nouveau package externe ni changement de version. Le générateur global utilise seulement la bibliothèque standard Python.

S05 ajoute nalgebra en dépendance de test directe de solvers pour la référence LU. Aucun nouveau package externe ni changement de version ; le solveur de production reste creux.

S06 ajoute la crate interne IO comme dépendance de test de FEM pour tester les entrées exécutables, sans dépendance de production inverse et sans nouveau package externe ni changement de version. Le générateur analytique utilise seulement la bibliothèque standard Python.

Licences déclarées par les packages ; inventaire à régénérer lors des changements de `Cargo.lock`. Les expressions observées incluent MIT, Apache-2.0, Unicode-3.0 ainsi que des alternatives Unlicense et Zlib. Retenir l’option MIT ou Apache-2.0 lorsqu’un choix est proposé, et respecter Unicode-3.0 lorsqu’elle est conjointe. La notation historique `MIT/Apache-2.0` nécessite de consulter les fichiers du package pour confirmer le choix. Le texte des licences et notices reste à inclure dans toute distribution concernée. Il ne s’agit pas d’un audit juridique ou de sécurité exhaustif.

| Package | Version résolue | Licence déclarée | Source |
| --- | --- | --- | --- |
| approx | 0.5.1 | Apache-2.0 | [package](https://crates.io/crates/approx/0.5.1) |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | [package](https://crates.io/crates/autocfg/1.5.1) |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | [package](https://crates.io/crates/bytemuck/1.25.2) |
| glam | 0.30.10 | MIT OR Apache-2.0 | [package](https://crates.io/crates/glam/0.30.10) |
| glam | 0.31.1 | MIT OR Apache-2.0 | [package](https://crates.io/crates/glam/0.31.1) |
| glam | 0.32.1 | MIT OR Apache-2.0 | [package](https://crates.io/crates/glam/0.32.1) |
| glam | 0.33.12 | MIT OR Apache-2.0 | [package](https://crates.io/crates/glam/0.33.12) |
| itoa | 1.0.18 | MIT OR Apache-2.0 | [package](https://crates.io/crates/itoa/1.0.18) |
| matrixmultiply | 0.3.11 | MIT/Apache-2.0 | [package](https://crates.io/crates/matrixmultiply/0.3.11) |
| memchr | 2.8.3 | Unlicense OR MIT | [package](https://crates.io/crates/memchr/2.8.3) |
| nalgebra | 0.35.0 | Apache-2.0 | [package](https://crates.io/crates/nalgebra/0.35.0) |
| nalgebra-macros | 0.3.0 | Apache-2.0 | [package](https://crates.io/crates/nalgebra-macros/0.3.0) |
| nalgebra-sparse | 0.12.0 | Apache-2.0 | [package](https://crates.io/crates/nalgebra-sparse/0.12.0) |
| num-bigint | 0.4.8 | MIT OR Apache-2.0 | [package](https://crates.io/crates/num-bigint/0.4.8) |
| num-complex | 0.4.6 | MIT OR Apache-2.0 | [package](https://crates.io/crates/num-complex/0.4.6) |
| num-integer | 0.1.47 | MIT OR Apache-2.0 | [package](https://crates.io/crates/num-integer/0.1.47) |
| num-rational | 0.4.2 | MIT OR Apache-2.0 | [package](https://crates.io/crates/num-rational/0.4.2) |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | [package](https://crates.io/crates/num-traits/0.2.19) |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [package](https://crates.io/crates/proc-macro2/1.0.107) |
| quote | 1.0.47 | MIT OR Apache-2.0 | [package](https://crates.io/crates/quote/1.0.47) |
| rawpointer | 0.2.1 | MIT/Apache-2.0 | [package](https://crates.io/crates/rawpointer/0.2.1) |
| safe_arch | 1.2.0 | Zlib OR Apache-2.0 OR MIT | [package](https://crates.io/crates/safe_arch/1.2.0) |
| serde | 1.0.229 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde/1.0.229) |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde_core/1.0.229) |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde_derive/1.0.229) |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde_json/1.0.154) |
| simba | 0.10.2 | Apache-2.0 | [package](https://crates.io/crates/simba/0.10.2) |
| syn | 2.0.119 | MIT OR Apache-2.0 | [package](https://crates.io/crates/syn/2.0.119) |
| syn | 3.0.9 | MIT OR Apache-2.0 | [package](https://crates.io/crates/syn/3.0.9) |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | [package](https://crates.io/crates/thiserror/2.0.21) |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | [package](https://crates.io/crates/thiserror-impl/2.0.21) |
| typenum | 1.20.1 | MIT OR Apache-2.0 | [package](https://crates.io/crates/typenum/1.20.1) |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [package](https://crates.io/crates/unicode-ident/1.0.27) |
| wide | 1.7.1 | Zlib OR Apache-2.0 OR MIT | [package](https://crates.io/crates/wide/1.7.1) |
| zmij | 1.0.23 | MIT | [package](https://crates.io/crates/zmij/1.0.23) |

## Vérification JSON Schema — développement seulement

`requirements-dev.txt` fixe les versions pour les contrôles Python. jsonschema 4.26.0 et les packages attrs, jsonschema-specifications, referencing et rpds-py déclarent MIT, vérifié dans leurs métadonnées installées. typing_extensions est requis conditionnellement avant Python 3.13 et fixé à 4.15.0 ; ses métadonnées PyPI déclarent PSF-2.0. Les textes de licences et notices sont à conserver avec toute redistribution concernée. Ces outils ne sont pas embarqués dans le moteur de simulation.
