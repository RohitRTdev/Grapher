import numpy as np
import matplotlib.pyplot as plt
from manifold import generateManifold
from plotGraph import plotGraph

def multi_gaussian(x, y, centers, sigma):
    z = np.zeros_like(x)
    for (cx, cy) in centers:
        z += np.exp(-((x - cx)**2 + (y - cy)**2) / (sigma**2))
    return z


def generate_gaussian_grid(n: int, extent: float = 2.0):
    x = np.linspace(-extent, extent, n)
    y = np.linspace(-extent, extent, n)
    X, Y = np.meshgrid(x, y)

    # Define 4 centers (corners of square)
    a = extent * 0.6
    centers = [
        (-a, -a),
        (-a,  a),
        ( a, -a),
        ( a,  a)
    ]

    sigma = 0.6

    Z = multi_gaussian(X, Y, centers, sigma)

    vertices = []
    values = []
    index_map = {}

    # Vertices + scalar field
    idx = 0
    for i in range(n):
        for j in range(n):
            xv = X[i, j]
            yv = Y[i, j]
            zv = Z[i, j]

            vertices.append([xv, yv, zv])

            # Here, our function is just the height map of the surface
            values.append(zv)

            index_map[(i, j)] = idx
            idx += 1

    # Create the edges 
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

    vertices, values, edges, X, Y, Z = generate_gaussian_grid(n)

    generateManifold(
        embedding_dim=3,
        vertices=vertices,
        values=values,
        edges=edges,
        file_path="gaussian2d.man"
    )

    print(f"Vertices: {len(vertices)}, Edges: {len(edges)}")

    plotGraph(X, Y, Z, vertices, edges)