"""Independent dense reference: exact affine polynomials and tensor weak form.

This script uses no FerraSim code, Voigt matrix or numerical linear algebra.
It is a separate calculation, not an independent human scientific review.
"""

import argparse
from fractions import Fraction as F
import json
from pathlib import Path

DESTINATION = Path(__file__).parent / "fixtures/tet4-reference.json"


def inverse_and_determinant(matrix):
    size = len(matrix)
    augmented = [list(row) + [F(i == j) for j in range(size)]
                 for i, row in enumerate(matrix)]
    determinant = F(1)
    for column in range(size):
        pivot = next(i for i in range(column, size) if augmented[i][column])
        if pivot != column:
            augmented[column], augmented[pivot] = augmented[pivot], augmented[column]
            determinant = -determinant
        value = augmented[column][column]
        determinant *= value
        augmented[column] = [entry / value for entry in augmented[column]]
        for row in range(size):
            if row != column:
                factor = augmented[row][column]
                augmented[row] = [a - factor * b for a, b in
                                  zip(augmented[row], augmented[column])]
    return [row[size:] for row in augmented], determinant


def reference(name, coordinates, young, poisson):
    polynomial = [[F(1)] + list(map(F, point)) for point in coordinates]
    inverse, determinant = inverse_and_determinant(polynomial)
    if determinant <= 0:
        raise ValueError("Reference vertices must have positive orientation")
    volume = determinant / 6
    gradients = [[inverse[axis + 1][node] for axis in range(3)] for node in range(4)]
    shear = F(young) / (2 * (1 + poisson))
    lame = F(young) * poisson / ((1 + poisson) * (1 - 2 * poisson))

    def tensor(i, j, k, ell):
        return (lame * (i == j) * (k == ell)
                + shear * ((i == k) * (j == ell) + (i == ell) * (j == k)))

    stiffness = [[F(0) for _ in range(12)] for _ in range(12)]
    for a in range(4):
        for b in range(4):
            for i in range(3):
                for k in range(3):
                    stiffness[3*a+i][3*b+k] = volume * sum(
                        tensor(i, j, k, ell) * gradients[a][j] * gradients[b][ell]
                        for j in range(3) for ell in range(3))
    return {"name": name, "coordinates": coordinates, "young": young,
            "poisson": str(poisson), "volume": str(volume),
            "gradients": [[str(value) for value in row] for row in gradients],
            "stiffness": [[str(value) for value in row] for row in stiffness]}


def document():
    skew = [[1, 2, -1], [3, 3, -1], [1, 5, 0], [2, 2, 3]]
    return {"reference_version": 1,
            "method": "Exact affine interpolation and C_ijkl tensor contraction",
            "dof_order": "node-major xyz", "gradients_order": "node then xyz",
            "cases": [
                reference("unit-steel", [[0,0,0],[1,0,0],[0,1,0],[0,0,1]],
                          210000000000, F(3,10)),
                reference("skew", skew, 150, F(1,4)),
                reference("skew-auxetic", skew, 150, F(-1,5)),
            ]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = json.dumps(document(), indent=2, sort_keys=True) + "\n"
    if args.check:
        if DESTINATION.read_text(encoding="utf-8") != generated:
            raise SystemExit("Dense TET4 reference differs from exact regeneration")
        print("Three dense references match exact rational regeneration.")
    else:
        DESTINATION.write_text(generated, encoding="utf-8")
        print(DESTINATION)
