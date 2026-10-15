# Traction uniforme d’un bloc — référence S06

Bloc SI 2×1×1, six TET4 ; E=1000 Pa, ν=0.25. Traction globale (10,0,0) Pa sur x=2. Appuis par composante : ux=0 à x=0, uy=0 à y=0, uz=0 à z=0 ; la contraction latérale reste libre.

```sh
cargo run -p ferrasim-cli --example solve_model --locked -- examples/uniaxial/model.json 10 2
```

Les arguments déclarent des échelles de 10 N et 2 m, avec moments à l’origine. Solution analytique : u=(0.01x,−0.0025y,−0.0025z), sigma_xx=10 Pa et énergie 0.1 J. Les autres contraintes sont nulles.

[Modèle](model.json), [maillage](mesh.msh) et [référence](../../validation/fixtures/uniaxial-reference.json) sont créés pour FerraSim et régénérés par le [script analytique](../../validation/generate_load_reference.py). Voir le [protocole, critères et limites](../../docs/LOADS-BALANCE.md). La revue scientifique indépendante reste ouverte ; cet exemple ne qualifie pas un domaine industriel. Aucun fichier de résultat n’est encore exporté.
