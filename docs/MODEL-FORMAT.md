# Contrat de modèle JSON 0.1

Statut : contrat implémenté en S01 ; import et liaison au maillage disponibles en S02 via une API distincte. Aucun calcul mécanique n’est encore exécuté.

Exemple : [model.json](../examples/minimal/model.json). Schéma pour éditeurs et outils : [JSON Schema 2020-12](../schemas/model-0.1.schema.json).

## Champs obligatoires

| Champ | Contrat |
| --- | --- |
| `schema_version` | Chaîne exacte `0.1` ; le brouillon `0.1-draft` n’est pas accepté |
| `units` | `"SI"` ou objet `{ "length": "mm", "force": "N" }`, par exemple |
| `mesh` | `path` relatif au fichier JSON, et catalogue `groups` |
| `materials` | Liste non vide de matériaux identifiés |
| `material_assignments` | Une affectation par groupe de volume déclaré |
| `boundary_conditions` | Liste de prescriptions sur groupes nodaux ; peut être vide à ce stade |
| `loads` | Liste de forces nodales ou tractions surfaciques ; peut être vide |
| `analysis` | `{ "type": "linear_static_3d" }` |
| `solver_settings` | Méthode `cg`, préconditionneur `jacobi`, `rtol`, `atol`, `max_iterations` |
| `outputs` | Liste `formats` non vide parmi `vtu`, `csv`, `json`, sans doublon |

Les champs inconnus, variantes non prises en charge, champs obligatoires absents et clés connues répétées sont rejetés. Aucun paramètre par défaut ne remplace une entrée physique manquante. Seules les composantes de déplacement peuvent être omises : elles restent libres.

## Unités

Avec SI : m, N et Pa. Avec un système cohérent personnalisé : longueur et force sont des labels explicites ; module et traction utilisent force/longueur². Le moteur ne convertit pas les valeurs, et ne vérifie pas la signification physique d’un label arbitraire. L’utilisateur est responsable de leur cohérence. Les labels doivent être non vides, sans espaces externes ni caractères de contrôle.

## Identifiants et groupes

Matériaux et groupes sont identifiés par chaînes non vides, sans espaces externes ni caractères de contrôle. Les noms Unicode sont autorisés. Les identifiants de matériaux sont uniques. Les groupes sont uniques dans chaque catégorie ; un même nom peut désigner un groupe nodal et sa surface d’origine.

```json
"mesh": {
  "path": "maillages/pièce.msh",
  "groups": {
    "nodes": ["base"],
    "surfaces": ["base", "loaded"],
    "volumes": ["solid"]
  }
}
```

Le catalogue déclare les sélections attendues, pas leurs membres. Une référence à un groupe absent du catalogue est rejetée en S01. S02 vérifie les noms et membres réels, les groupes vides utilisés, les nœuds absents, la couverture matériau des volumes et les prescriptions incompatibles sur groupes chevauchants. S04 construira les affectations et contraintes du système. Voir le [contrat de maillage](MESH-FORMAT.md).

Les identifiants nodaux sont des entiers positifs `u64`, écrits comme entiers dans le JSON ; ils ne sont pas des indices de tableau. Un nœud positif ne peut pas encore être vérifié contre un maillage non importé.

## Matériaux et affectations

Type initial : `isotropic_linear_elastic`, avec `id`, `young > 0` et `−1 < poisson < 0.5`. Les valeurs doivent être finies. Chaque groupe de volume déclaré doit avoir exactement une affectation vers un matériau existant. Deux affectations identiques du même volume sont également rejetées comme ambiguës.

La validation de plages d’entrée n’est pas une garantie de conditionnement, de pertinence de TET4 ou d’absence de débordement dans des calculs futurs. Les avertissements de quasi-incompressibilité et les contrôles des coefficients dérivés sont prévus avec le matériau en S03.

## Prescriptions

```json
{"node_group": "base", "components": {"x": 0.0, "z": 0.002}}
```

Une composante absente ou `null` reste libre. Au moins une composante doit être prescrite. Une valeur nulle ou non nulle est valide si elle est finie. Deux prescriptions de même groupe/composante doivent être exactement identiques ; sinon elles sont rejetées. Les prescriptions identiques sont conservées et seront fusionnées lors de construction des contraintes.

Les contradictions entre groupes différents partageant des nœuds sont contrôlées par la liaison modèle–maillage S02, pas par le parseur JSON seul. Un modèle sans contraintes peut réussir ces contrôles mais échouer ensuite pour singularité.

## Charges

```json
{"type":"nodal_force","node":4,"components":{"x":0.0,"y":0.0,"z":-1.0}}
```

```json
{"type":"surface_traction","surface_group":"loaded","components":{"x":0.0,"y":0.0,"z":-1000.0}}
```

Les trois composantes sont obligatoires et finies. Une force est appliquée à un seul nœud ; une traction est une force par aire intégrée sur une surface. Aucun partage implicite d’une résultante n’est effectué. Plusieurs enregistrements de charges sont additifs ; ils ne sont pas dédupliqués silencieusement. Leur assemblage est prévu en S06.

## Paramètres numériques

`rtol` et `atol` sont finis et non négatifs, et au moins l’un est strictement positif. `max_iterations` est un entier `u64` strictement positif. Le futur critère est `||b−Au||₂ ≤ atol + rtol*||b||₂`. Ces valeurs restent exprimées dans les unités effectives du système ; elles ne garantissent pas la convergence ni une précision physique donnée.

## Chemins et chargement

Dans le JSON, utiliser des chemins relatifs à séparateurs `/`. Les chemins absolus, lecteurs `C:`, antislashs, caractères de contrôle et chemins vides sont rejetés. Espaces internes, Unicode et `..` sont autorisés. Le fichier reste soumis aux règles de noms et droits de l’OS ; le contrat ne garantit pas qu’un fichier existe.

`ferrasim_io::load_model` canonicalise le fichier source et résout le chemin du maillage depuis son répertoire, indépendamment du répertoire courant. Le maillage n’est pas ouvert en S01. Des `..` peuvent donc référencer un fichier extérieur au dossier du modèle ; ce contrat n’est pas une sandbox de projet.

## API et erreurs

- `parse_model_json(&str)` : vérifie version, forme JSON et règles sémantiques.
- `Simulation::validate()` : vérifie aussi un modèle construit ou modifié directement en Rust.
- `model_to_json(&Simulation)` : valide avant sérialisation, pour empêcher qu’une valeur non finie devienne silencieusement `null`.
- `load_model(path)` : retourne modèle, chemin source et chemin de maillage résolu.
- `load_meshed_model(path)` : importe le maillage et vérifie les références réelles, selon le [contrat S02](MESH-FORMAT.md).

Les erreurs distinguent lecture fichier, JSON, version inconnue et validation sémantique. Les erreurs JSON exposent ligne/colonne ; les problèmes sémantiques sont collectés avec code stable, chemin de champ et message. La réussite de ces fonctions concerne uniquement le contrat du modèle.

Le JSON Schema est un outil de validation structurelle. Les références, unicités d’IDs métier, affectations, contradictions et détails lexicaux des nombres/labels sont contrôlés par l’API Rust. Un outil JSON Schema qui accepte un document ne dispense pas de ce contrôle.

## Démonstration

```sh
cargo run -p ferrasim-io --example inspect_model --locked -- examples/minimal/model.json
```

Ce programme d’exemple inspecte un modèle ; ce n’est pas la commande `ferrasim validate` du parcours MVP, prévue en S08. Aucun calcul FEM n’est lancé.
