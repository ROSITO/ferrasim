# Modèle proposé pour S01

`model.json` est un brouillon de contrat, pas un format déjà accepté par le moteur. Il montre les unités SI, chemins relatifs au modèle, groupes, affectations, déplacements et force nodale.

Le maillage est une fixture d’un tétraèdre, pas le démonstrateur de poutre et pas une validation de flexion. Le groupe de nœuds `base` est dérivé du triangle physique homonyme. La valeur `max_iterations` et les tolérances sont des paramètres illustratifs, pas des réglages universels.

La CLI S00 ne charge ni ne résout ce fichier. Le schéma et son contrôle des erreurs sont prévus en S01.
