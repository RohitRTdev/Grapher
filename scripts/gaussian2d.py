import numpy as np
import matplotlib.pyplot as plt
from manifold import generateManifold


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

    sigma = 0.5

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
            u = index_map[(i, j)]

            if j + 1 < n:
                edges.append((u, index_map[(i, j + 1)]))

            if i + 1 < n:
                edges.append((u, index_map[(i + 1, j)]))

    return vertices, values, edges, X, Y, Z


def plot_graph(X, Y, Z, vertices, edges):
    fig = plt.figure()
    ax_surface = fig.add_subplot(1, 2, 1, projection="3d")
    ax_wireframe = fig.add_subplot(1, 2, 2, projection="3d")

    ax_surface.plot_surface(X, Y, Z)
    ax_surface.set_title("Surface plot")

    for u, v in edges:
        x = [vertices[u][0], vertices[v][0]]
        y = [vertices[u][1], vertices[v][1]]
        z = [vertices[u][2], vertices[v][2]]

        ax_wireframe.plot(x, y, z, linewidth=0.5)

    xs = [v[0] for v in vertices]
    ys = [v[1] for v in vertices]
    zs = [v[2] for v in vertices]

    ax_wireframe.scatter(xs, ys, zs, s=5)

    ax_wireframe.set_title("Manifold 1d skeleton")

    plt.show()

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

    plot_graph(X, Y, Z, vertices, edges)