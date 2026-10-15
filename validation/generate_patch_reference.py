"""Exact global assembly and reduction, separate from the Rust/Voigt code.

Uses the S03 rational tensor reference, dense Python lists, and exact rational
elimination. A separate implementation is not independent human review.
"""
import argparse
from fractions import Fraction as F
import json
from pathlib import Path
from generate_tet4_reference import inverse_and_determinant, reference

DESTINATION = Path(__file__).parent / "fixtures/patch-reference.json"


def document():
    # Four boundary vertices, one interior node; deliberately unsorted sparse IDs.
    coordinates = [[0, 0, 0], [4, 0, 0], [0, 4, 0], [0, 0, 4], [1, 1, 1]]
    ids = [90, 7, 400, 23, 61]
    connectivities = [[4, 1, 2, 3], [0, 4, 2, 3], [0, 1, 4, 3], [0, 1, 2, 4]]
    cases = []
    for name, moduli in [("affine-patch", [150]*4), ("heterogeneous-forced", [150, 300, 75, 225])]:
        matrix = [[F(0) for _ in range(15)] for _ in range(15)]
        for connectivity, young in zip(connectivities, moduli):
            local = reference(name, [coordinates[i] for i in connectivity], young, F(1, 4))
            for a in range(12):
                for b in range(12):
                    matrix[3*connectivity[a//3]+a%3][3*connectivity[b//3]+b%3] += F(local["stiffness"][a][b])
        # All nine gradient entries and a translation, exact rational arithmetic.
        gradient = [[F(2,100), F(-1,100), F(3,100)],
                    [F(4,100), F(-3,100), F(1,100)],
                    [F(-2,100), F(5,100), F(1,100)]]
        translation = [F(1,10), F(-2,10), F(3,10)]
        affine = [translation[a] + sum(gradient[a][b]*point[b] for b in range(3))
                  for point in coordinates for a in range(3)]
        force = [F(0)]*15
        if name == "heterogeneous-forced":
            force = [F((-1)**i * (i+1), 10) for i in range(15)]
        prescribed = list(range(12))
        free = [12, 13, 14]
        reduced = [[matrix[i][j] for j in free] for i in free]
        rhs = [force[i] - sum(matrix[i][j]*affine[j] for j in prescribed) for i in free]
        inverse, _ = inverse_and_determinant(reduced)
        solution = [sum(inverse[i][j]*rhs[j] for j in range(3)) for i in range(3)]
        full = affine[:12] + solution
        residual = [sum(matrix[i][j]*full[j] for j in range(15)) - force[i] for i in range(15)]
        if name == "affine-patch":
            assert full == affine
        assert residual[12:] == [0, 0, 0]
        cases.append(dict(name=name, young=moduli,
                          stiffness=[[str(x) for x in row] for row in matrix],
                          forces=list(map(str, force)), prescribed=list(map(str, affine[:12])),
                          reduced=[[str(x) for x in row] for row in reduced],
                          rhs=list(map(str, rhs)), solution=list(map(str, full)),
                          residual=list(map(str, residual))))
    return dict(reference_version=1, method="Rational tensor assembly and exact dense elimination",
                node_ids=ids, coordinates=coordinates, connectivities=connectivities,
                poisson="1/4", gradient=[[str(x) for x in row] for row in gradient],
                cases=cases)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = json.dumps(document(), indent=2, sort_keys=True) + "\n"
    if args.check:
        if DESTINATION.read_text(encoding="utf-8") != generated:
            raise SystemExit("Global patch reference differs from exact regeneration")
        print("Two assembled references match exact rational regeneration.")
    else:
        DESTINATION.write_text(generated, encoding="utf-8")
        print(DESTINATION)
