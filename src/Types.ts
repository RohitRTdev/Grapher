type Node = {
  id: number;
  colorCode: number;
  x: number;
  y: number;
  z: number;
};

type Edge = {
  source: number;
  target: number;
};

export type Graph = {
  nodes: Node[];
  edges: Edge[];
};
