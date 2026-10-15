# Champs et exports de résultats — S07, format 1

`solve_static_results(mesh, model, origin, scales)` assemble et résout les charges du modèle, puis récupère les champs à partir de cette même solution. Le résultat `SimulationResults` est immuable : sa construction reste interne au chemin de résolution réussi. Un échec de CG, de récupération ou de plage numérique ne produit pas cet objet.

FEM calcule les champs ; IO les encode et les écrit. La dépendance de production IO → FEM ne forme pas de cycle de production ; FEM utilise IO seulement dans ses tests pour charger les fixtures. Aucun module FEM ne sérialise les résultats.

## Localisation et composantes

| Champ | Localisation | Composantes / unité cohérente |
| --- | --- | --- |
| Coordonnées, déplacement | Nœud, IDs externes triés | xyz ; longueur |
| Force appliquée, réaction | Nœud / composante | xyz ; force |
| Prescription | Nœud / composante | xyz ; longueur ; absence = libre |
| Déformation | Élément TET4, constante | xx, yy, zz, gamma_xy, gamma_yz, gamma_xz ; sans dimension |
| Contrainte de Cauchy | Élément TET4, constante | xx, yy, zz, tau_xy, tau_yz, tau_xz ; force/longueur² |
| Von Mises | Élément TET4, scalaire | force/longueur² |
| Volume et énergie | Élément TET4 | longueur³ et force×longueur |

Tous les champs utilisent le repère cartésien global et f64. Il n’existe pas de contrainte nodale brute ou de lissage dans cette API. Les réactions et prescriptions libres sont `None` dans l’API et `null` en JSON ; une réaction contrainte égale à zéro reste une valeur présente.

Von Mises est calculé à partir du tenseur de contrainte, avec normalisation préalable pour éviter les carrés de contraintes très grandes ou petites. La formule est celle des [conventions numériques](NUMERICAL-CONVENTIONS.md). Entrées non finies ou résultat non représentable sont rejetés. Un état hydrostatique donne zéro ; la déformation de cisaillement est bien la déformation d’ingénieur, sans facteur deux implicite dans l’export.

## VTU : XML UnstructuredGrid ASCII

`results_to_vtu` écrit des tétraèdres VTK de type 10. La connectivité Int64 est un indice de point interne commençant à zéro, et les offsets valent 4, 8, 12, etc. Les IDs externes restent des tableaux UInt64 séparés, y compris au-delà de 2⁵³. Aucune conversion de ces IDs en f64. Les IDs JSON sont aussi des entiers 64 bits ; leur lecteur doit préserver cette plage, ce qu’un simple Number JavaScript ne garantit pas.

- `PointData` : `NodeId`, `Displacement`, `AppliedForce`, `Reaction`, `PrescribedMask`, `PrescribedDisplacement`.
- `CellData` : `ElementId`, `MaterialIndex`, `StrainEngineering` (6), `StressVoigt` (6), `VonMises`, `Volume`, `ElasticEnergy`.
- `Points` : coordonnées originales non déformées. Le déplacement est un vecteur séparé, utilisable par un filtre de déformation du visualiseur.

Le masque de prescription comporte trois composantes UInt8 : 1 = prescrit, 0 = libre. Dans les tableaux VTU de réaction/prescription, une composante libre contient zéro et doit être interprétée avec ce masque. Les six composantes de tenseur sont des tableaux de champs ; elles ne sont pas déclarées comme des tenseurs VTK à neuf composantes. Le scalaire actif est `VonMises` et le vecteur actif `Displacement`.

`MaterialIndex` correspond au dictionnaire du manifeste/rapport, trié par identifiant matériau. Le VTU contient seulement des données numériques ; les noms utilisateur ne sont pas interpolés dans le XML. Les unités et conventions sont publiées dans le manifeste qui accompagne le fichier. Le format suit la [spécification VTK XML](https://docs.vtk.org/en/latest/vtk_file_formats/vtkxml_file_format.html).

## CSV UTF-8

Deux tables distinctes, sans conversion d’unité :

- `nodes.csv` : ID externe, coordonnées xyz, ux/uy/uz, fx/fy/fz, rx/ry/rz et déplacements prescrits xyz.
- `elements.csv` : ID externe, quatre IDs de nœuds, ID matériau, volume, six déformations, six contraintes, von Mises et énergie.

Les en-têtes nomment explicitement gamma pour les cisaillements de déformation et tau pour ceux de contrainte. Les composantes libres de réaction/prescription sont des cellules vides ; les zéros contraints sont écrits. Séparateur virgule, point décimal, fins de ligne LF, chaînes contenant virgule/guillemet correctement citées et guillemets doublés. Les valeurs f64 utilisent une représentation décimale qui permet leur relecture sans arrondi volontaire. Les unités du manifeste s’appliquent aux colonnes.

## Rapport JSON et provenance

`results_to_json` produit `result_version=1`. Il contient :

- Statut de convergence numérique et qualification scientifique indépendante encore ouverte.
- Version du moteur, OS/architecture, labels source fournis par l’appelant et éventuelle révision déclarée.
- Instantané exact du modèle parsé et du maillage validé : nœuds, tétraèdres, triangles, tags et groupes physiques.
- Unités, conventions, dictionnaire des matériaux et avertissements.
- Paramètres du solveur, itérations, résidu frais, résidu relatif et seuil.
- Origine/échelles des bilans et leurs valeurs complètes.
- Tous les champs nodaux et élémentaires, avec IDs externes.

Les instantanés décrivent les données utilisées, sans prétendre conserver les octets ou espaces du fichier source. Les labels source et la révision optionnelle sont déclaratifs ; aucune empreinte cryptographique ni attestation de compilation n’est annoncée. L’exemple conserve `declared_code_revision=null` plutôt que d’inventer une révision. Les temps et ressources non mesurés ne sont pas inventés non plus. Les labels d’unités cohérentes personnalisées sont conservés, sans conversion.

La feature `float_roundtrip` de serde_json est activée pour relire exactement les f64. Sans elle, un contrôle a observé une différence d’un dernier bit sur une petite réaction ; les tests exigent maintenant l’identité des nombres relus avec l’API. Aucune nouvelle version de package Rust n’est introduite.

## Publication des fichiers

`export_results(results, provenance, new_directory)` respecte `model.outputs.formats`. VTU produit `results.vtu`, CSV produit les deux tables et JSON produit `report.json`. `manifest.json` accompagne toujours les formats choisis : unités, conventions, matériaux, provenance déclarée, qualification ouverte, liste des fichiers et tailles en octets.

Le parent doit exister et le répertoire cible doit être absent, même vide. Les données sont encodées avant création du répertoire. Chaque fichier est créé sans remplacement et synchronisé. Le manifeste est écrit sous un nom temporaire puis publié en dernier, avec `status=complete`, après réussite des fichiers demandés.

Ce protocole n’est pas un renommage atomique de tout le répertoire : des fichiers intermédiaires peuvent être visibles pendant l’écriture. Les consommateurs doivent exiger le manifeste complet. Sur erreur d’écriture, le code retire les fichiers créés par l’appel puis tente de retirer son répertoire ; si le nettoyage échoue, l’absence de manifeste complet interdit de considérer le répertoire partiel comme réussi. Les répertoires préexistants sont rejetés sans modification. Les écritures concurrentes dans ce répertoire et les garanties de reprise après coupure ne sont pas qualifiées ici.

La commande CLI publique `solve` et la politique de sorties sont implémentées en [S08](CLI-API.md). L’exemple Rust S07 ne montre un succès qu’après résolution, récupération et publication complètes ; une entrée invalide ou non-convergence ne crée pas de destination.

## Vérifications et visualisation

Les tests FEM comparent les champs au bloc analytique S06 et à un cisaillement pur xz. Ils couvrent von Mises uniaxial, hydrostatique et les trois plans de cisaillement à plusieurs échelles, ainsi que les erreurs. Les tests IO contrôlent snapshots, nombres relus, unités personnalisées, labels avec virgules/guillemets, IDs > 2⁵³, masques, formats sélectionnés, Unicode et refus des destinations existantes.

Le [contrôle indépendant](../scripts/check_result_exports.py) relit le XML, les CSV, le JSON et le manifeste avec la bibliothèque standard Python. Il compare toutes les composantes exportées, la connectivité, les matériaux et les tailles. Le [contrôle analytique](../validation/verify_result_reference.py) impose une erreur normalisée maximale de 10⁻¹⁰ sur les champs uniaxiaux. Le [contrôle d’échec](../scripts/check_export_failure.py) vérifie absence de résultats après entrée invalide/non-convergence et préservation d’une destination existante.

Le [contrôle VTK natif](../scripts/check_vtk_results.py), optionnel pour le développement, ouvre réellement le VTU avec VTK 9.7.1, vérifie champs et connectivité puis peut le rendre. Il a été exécuté sur macOS ARM64 avec rendu hors écran, pas dans une GUI ParaView. Le [rendu inspecté](../validation/results/uniaxial-vtk.png) utilise les contraintes brutes par cellule et un déplacement amplifié ×10. La valeur de von Mises attendue est 10 Pa dans tous les éléments.

Les [exports de référence](../validation/results/uniaxial/manifest.json) sont un exemple observé sur macOS, pas une référence de chaînes décimales identiques entre OS. Les comparaisons physiques restent numériques dans les tolérances. L’[environnement VTK observé](../validation/vtk-environment.txt) est conservé ; ces packages Python ne sont pas nécessaires au moteur Rust ni à ses exports.

```sh
cargo run -p ferrasim-cli --example export_model --locked -- examples/uniaxial/model.json target/new-results 10 2
python3 scripts/check_result_exports.py target/new-results
python3 validation/verify_result_reference.py target/new-results
python3 scripts/check_export_failure.py
```

Pour le contrôle visuel optionnel, dans un venv séparé :

```sh
python -m pip install vtk==9.7.1
python scripts/check_vtk_results.py target/new-results --render target/vtk-preview.png
```

Les données et rendus sont créés pour FerraSim sous sa licence MIT. La revue scientifique par une personne distincte reste ouverte avant qualification ; les exports cohérents et leur lecture dans VTK ne remplacent pas la convergence de maillage ou la comparaison externe.
