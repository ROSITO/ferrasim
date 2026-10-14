# ADR 0003 — Import MSH et contrôles de topologie

Date : 14 octobre 2026. Statut : adopté pour S02.

## Choix

- Parsing MSH 2.2 ASCII dans IO, représentation et validation indépendante du format dans mesh.
- TRI3 et TET4 seulement ; autres topologies et sections non implémentées rejetées explicitement.
- Conservation des IDs, tags physiques/élémentaires et noms ; aucun renommage synthétique des tags non nommés.
- Seuil relatif de déterminant `10⁻¹²`, normalisé par la plus grande arête cubée ; tests d’échelles et cas analytiques. Ce seuil reste à relire avant qualification S03.
- Rejet des volumes inversés ; conservation du winding des surfaces et métadonnées extérieures déduites des volumes.
- Validation des incidences de faces et des surfaces réellement sélectionnées pour les charges.
- Dérivation des groupes nodaux par connectivité de région physique ; noms ambigus entre dimensions refusés.
- Couverture matériau obligatoire de tous les TET4 importés ; aucune sélection partielle implicite de volumes.
- Vérification anticipée des prescriptions contradictoires sur groupes chevauchants, maintenant que leurs membres sont connus.
- Rejet des nœuds inutilisés pour éviter leur intégration involontaire dans un système singulier.

## Conséquences

L’importeur accepte un sous-ensemble documenté, pas tous les maillages Gmsh. Un fichier valide comportant des lignes, points, données périodiques ou partitions devra être préparé pour ce sous-ensemble ou attendre une extension explicite. Les triangles internes sont disponibles pour les groupes, mais pas pour des charges de frontière.

Les checks géométriques ne constituent pas une vérification complète des intersections globales, ni une preuve de solvabilité mécanique. La [documentation](../MESH-FORMAT.md) et les erreurs exposent ces limites.
