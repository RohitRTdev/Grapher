import numpy as np
import matplotlib.pyplot as plt
from manifold import generateManifold
import umap

def embedding(u, v, w, t):
    x1 = np.cos(u)
    x2 = np.sin(u)
    x3 = np.cos(v + w)
    x4 = np.sin(v + t)
    x5 = np.sin(w + t)
    return [x1, x2, x3, x4, x5]


def scalar_field(u, v, w, t):
    return np.sin(u) + np.cos(v) + np.sin(w + t)


def generate_parametric_manifold(n: int):
    grid = np.linspace(-np.pi, np.pi, n)

    vertices = []
    values = []
    index_map = {}

    idx = 0

    for i, u in enumerate(grid):
        for j, v in enumerate(grid):
            for k, w in enumerate(grid):
                for l, t in enumerate(grid):
                    vert = embedding(u, v, w, t)
                    val = scalar_field(u, v, w, t)

                    vertices.append(vert)
                    values.append(val)

                    index_map[(i, j, k, l)] = idx
                    idx += 1

    edges = []

    for i in range(n):
        for j in range(n):
            for k in range(n):
                for l in range(n):
                    u_idx = index_map[(i, j, k, l)]

                    if i + 1 < n:
                        edges.append((u_idx, index_map[(i+1, j, k, l)]))
                    if j + 1 < n:
                        edges.append((u_idx, index_map[(i, j+1, k, l)]))
                    if k + 1 < n:
                        edges.append((u_idx, index_map[(i, j, k+1, l)]))
                    if l + 1 < n:
                        edges.append((u_idx, index_map[(i, j, k, l+1)]))

    return vertices, values, edges


def visualize_umap(vertices, values):
    reducer = umap.UMAP(n_components=3, init="random", n_neighbors=25, random_state=42)
    embedding = reducer.fit_transform(vertices)

    fig = plt.figure()
    ax = fig.add_subplot(projection='3d')

    sc = ax.scatter(
        embedding[:, 0],
        embedding[:, 1],
        embedding[:, 2],
        c=values,
        s=5
    )

    fig.colorbar(sc)
    ax.set_title("4-manifold in R^5 (UMAP projection)")

    plt.show()


if __name__ == "__main__":
    n = 8  

    vertices, values, edges = generate_parametric_manifold(n)

    generateManifold(
        embedding_dim=5,
        vertices=vertices,
        values=values,
        edges=edges,
        file_path="manifold4d_in_5d.man"
    )

    print(f"Vertices: {len(vertices)}, Edges: {len(edges)}")

    visualize_umap(vertices, values)