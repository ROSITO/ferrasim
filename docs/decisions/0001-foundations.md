# ADR 0001 — Fondations du MVP

Date : 14 octobre 2026. Statut : adopté pour l’implémentation S00 ; conventions scientifiques à relire indépendamment avant S03.

## Choix

- Licence MIT pour le dépôt, pour permettre usage, modification et intégration avec attribution. Les droits et obligations des dépendances restent distincts.
- Rust 1.96.0 fixé par Rustup, édition 2024 et resolver 3 du workspace.
- Cinq bibliothèques et une CLI, sans couche graphique ou orchestration distante.
- Précision `f64`, SI pour les exemples et pas de conversions automatiques dans le MVP.
- Dépendances de production choisies : Serde/serde_json, thiserror, nalgebra et nalgebra-sparse.
- Matrices élémentaires denses ; assemblage global creux CSR issu de triplets COO. Le CG/Jacobi sera implémenté et vérifié en S05 ; la bibliothèque creuse n’est pas réputée fournir à elle seule ce solveur.
- CLI sans framework en S00 ; clap est candidat pour S08, non ajouté tant qu’il n’est pas utilisé.
- Aucune dépendance BLAS/MPI/GPU, aucun nightly requis.

## Alternatives et raisons

| Domaine | Retenu | Alternative examinée | Raison / limite |
| --- | --- | --- | --- |
| Sérialisation | Serde et JSON | Parseur maison, YAML | Contrats Rust, JSON versionné ; validation physique toujours à écrire |
| Algèbre dense | nalgebra | Tableaux maison, faer | Matrices élémentaires et référence dense ; performance globale à mesurer plus tard |
| Algèbre creuse | nalgebra-sparse | sprs, stockage maison | COO/CSR compatibles avec nalgebra ; évite d’écrire le conteneur initial ; choix révisable après benchmark |
| Erreurs | thiserror | Chaînes, anyhow partout | Erreurs typées dans le moteur ; pas de dépendance de la physique à la CLI |
| Arguments CLI | std en S00 | clap | Deux flags informatifs seulement ; parser métier reporté à S08 |

L’alternative sprs n’a pas été adoptée ; son audit complet n’a pas été réalisé. Aucune affirmation de compatibilité de licence ou de performance n’est faite pour les alternatives non intégrées.

## Versions examinées et licences déclarées

| Dépendance | Version examinée | Licence déclarée | Source primaire |
| --- | --- | --- | --- |
| nalgebra | 0.35.0 | Apache-2.0 | [package](https://crates.io/crates/nalgebra/0.35.0) |
| nalgebra-sparse | 0.12.0 | Apache-2.0 | [package](https://crates.io/crates/nalgebra-sparse/0.12.0) |
| serde | 1.0.229 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde/1.0.229) |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | [package](https://crates.io/crates/serde_json/1.0.154) |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | [package](https://crates.io/crates/thiserror/2.0.21) |
| clap, candidat seulement | 4.6.7 | MIT OR Apache-2.0 | [package](https://crates.io/crates/clap/4.6.7) |

`Cargo.lock` conserve les versions effectivement résolues, y compris transitives. L’[inventaire](../DEPENDENCIES.md) est extrait des métadonnées Cargo. Les licences permissives examinées ne nécessitent pas de modifier la licence du code FerraSim ; les notices et textes tiers devront être conservés lors de distribution de binaires. L’inventaire déclaratif ne constitue pas un audit exhaustif du code ou de la sécurité des dépendances.

## Références de configuration

[Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Rustup et toolchain de projet](https://rust-lang.github.io/rustup/overrides.html), [nalgebra](https://nalgebra.rs/docs/).

## Décisions ouvertes

| Décision | Responsable attendu | Échéance de décision | État |
| --- | --- | --- | --- |
| Schéma JSON final et erreurs | Mainteneur modèle | Revue d’entrée S01 | Implémenté en S01, voir [ADR 0002](0002-model-contract.md) |
| Seuil géométrique relatif de dégénérescence | Référent numérique | Avant qualification S03 | Seuil initial 10⁻¹² implémenté et testé en S02 ; revue indépendante ouverte |
| Avertissement quasi-incompressibilité | Référent matériau | Avant sortie S03 | Intervalle physique fixé ; seuil d’avertissement à documenter |
| Solveur de comparaison et environnement | Référent validation | Préparation S09, obligatoire avant sortie S10 | CalculiX proposé ; installation et version non qualifiées |
| Personne de revue scientifique indépendante | Responsable projet | Avant sortie scientifique S03 | À désigner ; S00 ne présume pas cette revue acquise |
| Capacité réelle et calendrier MVP | Responsable projet | Revue S00 puis S03 | Aucune équipe ni date de démarrage future confirmée |

Les échéances sont des portes de sprint, les dates calendaires restant à fixer lors de leur lancement. Les responsabilités sont des rôles nécessaires, pas des affectations à des personnes déjà disponibles.
