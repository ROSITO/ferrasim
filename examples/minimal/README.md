# Modèle minimal 0.1

`model.json` est accepté par les fonctions de lecture et validation du contrat S01. Il montre les unités SI, le catalogue de groupes, les chemins relatifs au modèle, les affectations, les déplacements et une force nodale. Voir le [contrat JSON](../../docs/MODEL-FORMAT.md).

Le maillage est une fixture d’un tétraèdre, pas le démonstrateur de poutre et pas une validation de flexion. Le groupe de nœuds `base` est dérivé du triangle physique homonyme. La valeur `max_iterations` et les tolérances sont des paramètres illustratifs, pas des réglages universels.

La CLI ne charge ni ne résout encore ce fichier. `inspect_model` vérifie le JSON seul ; `inspect_mesh` importe le maillage et vérifie les références en S02. Voir le [contrat maillage](../../docs/MESH-FORMAT.md). Aucun calcul FEM n’est exécuté, et ces contrôles ne garantissent pas la solvabilité mécanique.
