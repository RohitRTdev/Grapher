type Node = {
  id: number;
  name: string;
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
