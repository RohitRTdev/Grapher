import struct
from typing import List, Tuple

def generateManifold(
        embedding_dim: int,
        vertices: List[List[float]],
        values: List[float],
        edges: List[Tuple[int, int]],
        file_path: str
        ):

    if embedding_dim <= 0:
        raise ValueError("Embedding dimension must be positive")

    num_vertices = len(vertices)
    num_edges = len(edges)

    if num_vertices != len(values):
        raise ValueError("Every vertex must have an associated function value")

    for v in vertices:
        if len(v) != embedding_dim:
            raise ValueError(
                f"Vertex {v} does not match embedding dim {embedding_dim}"
            )

    for u, v in edges:
        if not (0 <= u < num_vertices and 0 <= v < num_vertices):
            raise ValueError(f"Invalid edge ({u}, {v})")

    # Remove duplicate undirected edges
    edge_set = set()
    for u, v in edges:
        edge = tuple(sorted((u, v)))
        edge_set.add(edge)
    edges = list(edge_set)
    num_edges = len(edges)

    with open(file_path, "wb") as f:
        # Header
        f.write(struct.pack("<III", embedding_dim, num_vertices, num_edges))

        # Vertices
        for v, value in zip(vertices, values):
            # We store the function value associated with this vertex as final component
            f.write(struct.pack(f"<{embedding_dim + 1}d", *v, value))

        # Edges
        for u, v in edges:
            f.write(struct.pack("<II", u, v))



# Test
if __name__ == "__main__":
    vertices = [
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0]
    ]

    values = [
        3.0,
        0.0,
        -2.5
    ]

    edges = [
        (0, 1),
        (1, 2),
        (1, 0)  
    ]

    generateManifold(2, vertices, values, edges, "triangle.man")