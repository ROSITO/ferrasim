# S09 — Protocole v1, fixé avant mesure

Poutre pleine : L=1 m, b=h=0,05 m, E=210 GPa, ν=0,3. Encastrement complet à x=0 ; traction uniforme globale t_z=−40000 Pa à x=L, résultante −100 N. Autres faces libres.

Référence Euler–Bernoulli : −FL³/(3EI)=−0,0003047619047619 m, I=bh³/12. Observable : moyenne surfacique de u_z sur la face chargée ; intégration exacte du champ linéaire par triangle. Les triangles de cette famille ont tous la même aire. Cette référence de poutre élancée (L/h=20) reste une approximation du solide 3D ; les contraintes singulières à l'encastrement ne constituent pas un critère.

Famille structurée emboîtée : nx=20m, ny=nz=m, six TET4 par cellule, diagonale commune cohérente. Niveaux initiaux m=2,4,8 ; raffiner davantage si nécessaire, sans modifier le seuil. CG/Jacobi, rtol=10⁻¹⁰, atol=10⁻¹², budget 30000 ; échelles explicites F=100 N, L=1 m, origine nulle.

Acceptation : au moins trois niveaux, erreur décroissante, erreur finale ≤5 %. Bilans force, moment, résidu libre et énergie ≤10⁻⁶. Comparaison supplémentaire du niveau fin avec rtol=10⁻¹² : variation de flèche relative ≤10⁻⁶ pour séparer erreur algébrique et discrétisation. Un échec suspend la livraison et impose investigation ; aucun assouplissement a posteriori.

Mesure : binaire release, temps mural CLI complet (lecture, assemblage, calcul, récupération, export), une exécution par niveau ; pic RSS par processus via wait4 sur macOS/Linux, indisponible sous Windows. Empreintes SHA256 du binaire et des entrées ; données brutes dans le répertoire généré. Ces mesures ne sont pas une promesse de performance.

Reproduction : `cargo build --release --locked`, puis `python3 validation/run_cantilever.py --output target/s09-study`. Le répertoire doit être neuf. La qualification scientifique indépendante reste ouverte ; comparaison avec un solveur externe prévue en S10.

Le mode `--generate-only` prépare seulement les entrées. Le mode `--diagnostic-rtol 1e-8` est réservé à l'investigation et ne peut valider S09. L'échec du protocole v1 et les mesures diagnostiques sont publiés dans le [rapport S09](../docs/sprints/S09.md). La formule de poutre et ses hypothèses sont présentées dans le [cours universitaire de micromécanique](https://pastewka.github.io/Micromechanics/lecture/chapter04.html).
