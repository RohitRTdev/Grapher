import matplotlib.pyplot as plt

def plotGraph(X, Y, Z, vertices, edges):
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
