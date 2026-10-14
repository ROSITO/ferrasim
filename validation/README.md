# Protocole de validation scientifique

Le [registre](cases.toml) fixe les cas, métriques et tolérances avant calcul. **Tous les cas sont planifiés ; aucun résultat FEM n’est encore produit.** Le contrôle Python S00 vérifie le registre et les fixtures, pas la précision d’un solveur.

Chaque cas devra conserver entrées, référence indépendante, environnement, observable, échelles de normalisation, résultats et état passé/échoué. Les références sont choisies avant examiner les résultats. Les valeurs proches de zéro utilisent une échelle absolue ; les singularités ne sont pas des valeurs de comparaison ponctuelles valides.

## Protocole par famille

| Cas | Données et référence à préparer | Preuve attendue |
| --- | --- | --- |
| Symétrie et modes rigides | TET4 bien conditionné ; six champs rigides ; expression BᵀCB | Erreur normalisée ≤ 10⁻¹⁰ |
| Patch affine | Maillage multiélément et déplacement affine sur frontière | Déplacements et contraintes ≤ 10⁻⁸ |
| Traction uniaxiale | Barre, traction uniforme, contraintes supprimant seulement les modes rigides et permettant contraction latérale | Erreurs ≤ 10⁻⁶ |
| Prescriptions non nulles | Petit système et résolution dense indépendante | Erreur ≤ 10⁻⁸ |
| Traction surfacique | Triangle simple, aire et vecteur de traction connus | Résultante ≤ 10⁻¹⁰ |
| Équilibre | Charges, réactions et origine des moments | Bilan normalisé ≤ 10⁻⁶ |
| Poutre | Protocole ci-dessous et trois raffinements minimum | Convergence et erreur de déplacement ≤ 5 % selon PRD |
| Comparaison externe | Même maillage, matériau, forces et prescriptions ; solveur/version archivés | Déplacements/contraintes élémentaires ≤ 10⁻⁵ |
| Entrées invalides | Références absentes, groupes vides utilisés, dégénérescence, prescriptions contradictoires et singularité de référence | Rejet ou échec explicite, sans succès de calcul |

## Démonstrateur de poutre proposé

Poutre prismatique selon x : longueur 1 m, largeur 0,05 m selon y, hauteur 0,05 m selon z ; `E=210e9 Pa`, `ν=0.3`. Face x=0 encastrée ; résultante `Fz=−100 N` distribuée uniformément sur x=L. Aucune force concentrée sur un coin.

Observable : moyenne surfacique du déplacement z de la face chargée. Moment quadratique de référence pour cette flexion : `I = b*h^3/12`. Valeur Euler–Bernoulli : `−F_abs*L^3/(3*E*I) ≈ −0,000304761905 m`.

La formule est une référence de poutre approchée, pas une solution exacte du solide 3D. En S09, préparer au moins trois maillages documentés, publier erreur/déplacement/temps/ressources et distinguer discrétisation de l’écart de modèle. Le seuil de 5 % n’est pas un critère sur une contrainte singulière à l’encastrement. Si les hypothèses rendent cette comparaison inadaptée, utiliser une référence solide 3D indépendante convergée, comme autorisé par le PRD, et publier la justification.

## Sources et droits

- [A. F. Bower — élastostatique FEM](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm) : formulation et vérification théorique. Aucun code de cette source n’est copié.
- [Gmsh — manuel](https://gmsh.info/doc/texinfo/gmsh.html) : contrat du format MSH.
- [CalculiX — site du projet](https://www.calculix.de/) : candidat pour la comparaison externe ; installation et version à qualifier en S10.
- [Conventions FerraSim](../docs/NUMERICAL-CONVENTIONS.md) et [PRD MVP](../docs/PRD.md) : choix, tolérances et règles de remplacement de référence.

La fixture `tet4-positive.msh` et le brouillon JSON sont créés pour FerraSim et distribués sous la licence du dépôt. Les autres données à ajouter doivent porter provenance et droits de redistribution.
