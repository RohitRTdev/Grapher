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

export type Stats = {
  time: number;
  memory: number;
  accuracy: number;
  f1score: number;
}

export type Graph = {
  nodes: Node[];
  edges: Edge[];
};

export type FinalResult = {
  graph: Graph;
  time: number;
  memory: number;
  accuracy: number;
  f1score: number;
  is_vtk: boolean;
};

