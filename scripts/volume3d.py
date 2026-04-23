import numpy as np
from manifold import generateManifold

# This time, generate a 3 manifold


def scalar_field(x, y, z):
    # Combination of Gaussian + oscillations.
    r2 = x**2 + y**2 + z**2
    return np.exp(-r2) * np.cos(2*x) * np.cos(2*y) * np.cos(2*z)


def generate_3d_grid(n: int, extent: float = 2.0):
    xs = np.linspace(-extent, extent, n)
    ys = np.linspace(-extent, extent, n)
    zs = np.linspace(-extent, extent, n)

    vertices = []
    values = []
    index_map = {}

    idx = 0
    for i in range(n):
        for j in range(n):
            for k in range(n):
                x = xs[i]
                y = ys[j]
                z = zs[k]

                val = scalar_field(x, y, z)

                vertices.append([x, y, z])
                values.append(val)

                index_map[(i, j, k)] = idx
                idx += 1

    edges = []

    for i in range(n):
        for j in range(n):
            for k in range(n):
                u = index_map[(i, j, k)]

                # axis-aligned neighbors
                if i + 1 < n:
                    edges.append((u, index_map[(i+1, j, k)]))
                if j + 1 < n:
                    edges.append((u, index_map[(i, j+1, k)]))
                if k + 1 < n:
                    edges.append((u, index_map[(i, j, k+1)]))

                # face diagonals
                if i + 1 < n and j + 1 < n:
                    edges.append((u, index_map[(i+1, j+1, k)]))
                if i + 1 < n and k + 1 < n:
                    edges.append((u, index_map[(i+1, j, k+1)]))
                if j + 1 < n and k + 1 < n:
                    edges.append((u, index_map[(i, j+1, k+1)]))

                # main diagonal
                if i + 1 < n and j + 1 < n and k + 1 < n:
                    edges.append((u, index_map[(i+1, j+1, k+1)]))

    return vertices, values, edges

if __name__ == "__main__":
    n = 25  

    vertices, values, edges = generate_3d_grid(n)

    generateManifold(
        embedding_dim=3,
        vertices=vertices,
        values=values,
        edges=edges,
        file_path="gaussian3d.man"
    )

    print(f"Vertices: {len(vertices)}, Edges: {len(edges)}")