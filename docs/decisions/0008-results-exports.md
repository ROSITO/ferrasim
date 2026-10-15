# ADR 0008 — Champs immuables et exports à manifeste final

Date : 15 octobre 2026. Statut : implémenté et vérifié localement ; CI multiplateforme à consigner, revue scientifique indépendante ouverte.

## Décision

FEM construit un résultat immuable uniquement après résolution réussie et récupération des champs. Les contraintes et déformations sont constantes par TET4, en ordre xx/yy/zz/xy/yz/xz avec cisaillement d’ingénieur ; aucun lissage nodal. IO dépend de FEM pour encoder ces résultats, sans cycle de production.

VTU conserve les IDs UInt64 à côté d’une connectivité Int64 interne. CSV sépare nœuds et éléments ; JSON conserve snapshots, unités, paramètres, résidus et bilans. La lecture exacte des f64 est activée dans serde_json pour préserver les valeurs après relecture.

Les formats demandés sont écrits dans un nouveau répertoire et le manifeste de complétude est publié en dernier. Les destinations existantes sont refusées. La CLI complète est différée à S08 ; aucun faux succès après erreur de solveur ou d’écriture.

## Conséquences

Les snapshots et chaînes d’export consomment de la mémoire supplémentaire ; pas de compression, streaming ou qualification HPC. Les métadonnées source sont déclaratives et ne constituent pas une signature cryptographique. Une complétude de fichiers n’est pas une qualification scientifique.

Le XML est vérifié avec des lecteurs indépendants puis ouvert et rendu dans VTK natif sur macOS. Le protocole de répertoire partiel et ses limites sont documentés dans le [contrat](../RESULTS-FORMAT.md). Voir le [rapport S07](../sprints/S07.md).
