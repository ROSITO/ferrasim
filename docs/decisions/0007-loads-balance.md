# ADR 0007 — Forces consistantes et bilans sur le système original

Date : 15 octobre 2026. Statut : implémenté et vérifié sur Windows, macOS et Linux, revue scientifique indépendante ouverte.

## Décision

L’assemblage S06 intègre une fois les enregistrements de charges du modèle : force directe au nœud externe et traction vectorielle globale uniforme par triangle, avec A/3 à chaque sommet. Les enregistrements explicites se superposent ; aucune redistribution ou déduplication silencieuse. Les frontières internes et sélections invalides sont rejetées avant intégration.

Le système immuable conserve coordonnées et forces dans son ordre interne. La réduction des charges du modèle et leur résolution sont des chemins explicites, distincts de l’API qui accepte un vecteur alternatif. Les bilans du système utilisent uniquement ses charges stockées.

Les réactions viennent de K_original u − F_original sur les composantes prescrites ; le résidu libre reste séparé. Origine des moments et échelles positives sont explicites. Le bilan d’énergie inclut u_c^T R_c ; les diagnostics ne cachent pas de seuil d’acceptation ou de correction d’arrondi.

## Conséquences

Deux vecteurs de données supplémentaires (forces et coordonnées) restent attachés au système pour éviter de mêler des maillages ou modèles incompatibles. Les diagnostics peuvent signaler un mauvais champ sans le déclarer convergé. Les cas extrêmes non représentables échouent sans résultat partiel.

Le bloc uniaxial analytique à six TET4 vérifie les chargements, la solution et les bilans avant S07. Les champs exportés et la CLI complète restent à leurs sprints. Voir le [contrat](../LOADS-BALANCE.md) et le [rapport S06](../sprints/S06.md). La revue scientifique indépendante reste nécessaire avant qualification.
