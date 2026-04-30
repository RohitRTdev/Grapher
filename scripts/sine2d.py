import numpy as np
from manifold import generateManifold
from plotGraph import plotGraph

def sine_function(x, y):
    return np.sin(2*x) + np.sin(2*y)

def generate_wave_grid(n: int, extent: float = 2.0):
    x = np.linspace(-extent, extent, n)
    y = np.linspace(-extent, extent, n)
    X, Y = np.meshgrid(x, y)

    Z = sine_function(X, Y)

    vertices = []
    values = []
    index_map = {}

    idx = 0
    for i in range(n):
        for j in range(n):
            xv = X[i, j]
            yv = Y[i, j]
            zv = Z[i, j]

            vertices.append([xv, yv, zv])
            values.append(zv)

            index_map[(i, j)] = idx
            idx += 1

    edges = []
    for i in range(n):
        for j in range(n):
            is_within_y = is_within_x = False
            u = index_map[(i, j)]

            if j + 1 < n:
                edges.append((u, index_map[(i, j + 1)]))
                is_within_x = True

            if i + 1 < n:
                edges.append((u, index_map[(i + 1, j)]))
                is_within_y = True

            if is_within_x and is_within_y:
                edges.append((u, index_map[(i + 1, j + 1)]))

            if is_within_y and j > 0:
                edges.append((u, index_map[(i + 1, j - 1)]))

    return vertices, values, edges, X, Y, Z

if __name__ == "__main__":
    n = 60

    vertices, values, edges, X, Y, Z = generate_wave_grid(n, np.pi)

    generateManifold(
        embedding_dim=3,
        vertices=vertices,
        values=values,
        edges=edges,
        file_path="sine2d.man"
    )

    print(f"Vertices: {len(vertices)}, Edges: {len(edges)}")

    plotGraph(X, Y, Z, vertices, edges)
