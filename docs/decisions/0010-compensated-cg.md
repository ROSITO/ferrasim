# ADR 0010 — Produits compensés et mises à jour CG

Statut : accepté pour l'implémentation ; qualification scientifique indépendante ouverte.

## Problème

S09 révèle un écart entre résidu récursif et résidu recalculé sur une poutre élancée. Accumuler des produits de signes opposés peut perdre les termes faibles. Accumuler les incréments de déplacement peut également perdre les petites corrections. Augmenter seulement le budget ne corrige pas ces arrondis.

## Décision

Les produits CSR et les produits scalaires CG utilisent une accumulation de Neumaier, complétée par l'erreur de produit calculée avec `mul_add(a,b,-a*b)`. Les déplacements utilisent une mise à jour compensée ; le résidu récursif est mis à jour par FMA. Lorsqu'un redémarrage remplace le résidu récursif par `b-Ax`, la compensation des déplacements est remise à zéro : le nouveau système de correction porte sur l'itéré arrondi effectivement retournable.

Le succès exige toujours le résidu recalculé du vecteur `f64` public sous le seuil demandé. Aucun seuil n'est modifié implicitement, aucun échec n'est transformé en succès. Les références rationnelles, résidus frais et bilans physiques restent distincts. Une FMA et une compensation ne constituent pas une arithmétique de précision arbitraire ni une garantie de convergence pour toute tolérance.

## Vérification et limites

Deux tests binaires exacts vérifient séparément la perte d'un terme faible entre deux grands termes opposés et la perte de l'erreur d'un produit arrondi. Les tests CG existants recalculent le résidu du vecteur retourné. La poutre m=4 constitue une régression d'intégration supplémentaire. L'exemple `audit_residual` exporte le système réduit pour un audit externe à 100 chiffres avec la bibliothèque Python Decimal ; il ne crée pas un rapport de simulation.

Les entrées de matrice, les déplacements publics et les exports restent en `f64`. La somme des contributions d'assemblage et les calculs locaux FEM ne sont pas convertis en précision supérieure. Une tolérance trop exigeante peut toujours échouer explicitement. Les comparaisons Decimal concernent le système arrondi assemblé, pas une solution continue exacte ni une validation indépendante des rigidités.
