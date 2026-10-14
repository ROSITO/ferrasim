# Modèle minimal 0.1

`model.json` est accepté par les fonctions de lecture et validation du contrat S01. Il montre les unités SI, le catalogue de groupes, les chemins relatifs au modèle, les affectations, les déplacements et une force nodale. Voir le [contrat JSON](../../docs/MODEL-FORMAT.md).

Le maillage est une fixture d’un tétraèdre, pas le démonstrateur de poutre et pas une validation de flexion. Le groupe de nœuds `base` est dérivé du triangle physique homonyme. La valeur `max_iterations` et les tolérances sont des paramètres illustratifs, pas des réglages universels.

La CLI ne charge ni ne résout encore ce fichier ; utiliser le programme Rust `inspect_model` décrit dans le contrat. L’import du maillage arrive en S02, et la physique plus tard. Un contrat valide ne garantit pas un maillage valide ou un calcul solvable.
