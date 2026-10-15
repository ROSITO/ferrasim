# ADR 0006 — CG/Jacobi et succès fondé sur le résidu recalculé

Date : 15 octobre 2026. Statut : implémenté et vérifié sur Windows, macOS et Linux, revue scientifique indépendante ouverte.

## Décision

S05 implémente CG/Jacobi en f64 sur le CSR de S04, sans matrice dense en production. Départ nul, tolérances et budget explicites issus du modèle. La matrice est contrôlée pour symétrie et diagonale positive utilisable ; ces vérifications ne sont pas une certification de rang ni de définie-positivité.

Le succès exige un résidu `b−Kx` recalculé satisfaisant `atol+rtol*norm(b)`. Un résidu récursif trompeur déclenche un redémarrage dans le budget restant. Les échecs utilisent une branche `Err` avec statut et diagnostics, sans solution exportable comme réussie. Le second membre nul et le système 0×0 sont explicitement traités.

## Conséquences

Un produit CSR supplémentaire par itération et plusieurs vecteurs temporaires facilitent la vérification ; pas de promesse de performance industrielle à ce stade. Les ruptures numériques sont signalées sans régularisation ou repli automatique. La détection exhaustive des singularités n’est pas annoncée.

La LU dense reste réservée aux tests. Le corpus comprend les références FEM S04 et un cas singulier exact ; les exécutables d’échec ne doivent produire ni stdout de succès ni fichiers. L’intégration des charges JSON demeure S06 et les exports S07.

Voir le [contrat et protocole](../SOLVER-CG.md) et le [rapport S05](../sprints/S05.md). Une revue indépendante reste nécessaire avant qualification scientifique.
