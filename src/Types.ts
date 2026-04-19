type Node = {
  id: number;
  color_code: number;
  fn_val: number;
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
