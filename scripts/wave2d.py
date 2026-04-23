import numpy as np
import matplotlib.pyplot as plt
from manifold import generateManifold


def wave_function(x, y):
    r2 = x**2 + y**2

    # central bump + oscillations
    return np.exp(-r2) * np.cos(3 * x) * np.sin(3 * y)


def generate_wave_grid(n: int, extent: float = 2.0):
    x = np.linspace(-extent, extent, n)
    y = np.linspace(-extent, extent, n)
    X, Y = np.meshgrid(x, y)

    Z = wave_function(X, Y)

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


def plot_graph(X, Y, Z, vertices, edges):
    fig = plt.figure()
    ax_surface = fig.add_subplot(1, 2, 1, projection="3d")
    ax_wireframe = fig.add_subplot(1, 2, 2, projection="3d")

    ax_surface.plot_surface(X, Y, Z)
    ax_surface.set_title("Wave Surface")

    for u, v in edges:
        x = [vertices[u][0], vertices[v][0]]
        y = [vertices[u][1], vertices[v][1]]
        z = [vertices[u][2], vertices[v][2]]

        ax_wireframe.plot(x, y, z, linewidth=0.5)

    xs = [v[0] for v in vertices]
    ys = [v[1] for v in vertices]
    zs = [v[2] for v in vertices]

    ax_wireframe.scatter(xs, ys, zs, s=5)

    ax_wireframe.set_title("1-skeleton")

    plt.show()


if __name__ == "__main__":
    n = 60

    vertices, values, edges, X, Y, Z = generate_wave_grid(n)

    generateManifold(
        embedding_dim=3,
        vertices=vertices,
        values=values,
        edges=edges,
        file_path="wave2d.man"
    )

    print(f"Vertices: {len(vertices)}, Edges: {len(edges)}")

    plot_graph(X, Y, Z, vertices, edges)