import { useState, useRef, useEffect, useMemo } from "react";
import type {Graph, Stats} from './Types.ts'
import { invoke } from "@tauri-apps/api/core";
import { getGraphData, fetchData, fetchMemoryFormattedString, fetchLastGraph } from "./UI.ts";
import file_open_img from './assets/file-open.png';
import reset_img from './assets/reset.png';
import show_img from './assets/show.png';
import hide_img from './assets/hide.png';
import ForceGraph3D from "react-force-graph-3d";

export default function App() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState<Stats>({
                              time: 0,
                              memory: 0,
                              accuracy: 0
                            });

  
  const [disabled, setDisabled] = useState<boolean>(true);
  const [extImg, setExtImg] = useState<string>(show_img);
  const graphRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  
  // Watch the container for size changes
  useEffect(() => {
    const currentContainer = containerRef.current;
    if (!currentContainer) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setDimensions({
          width: entry.contentRect.width,
          height: entry.contentRect.height
        });
      }
    });

    resizeObserver.observe(currentContainer);

    // Cleanup observer on unmount
    return () => {
      resizeObserver.unobserve(currentContainer);
    };
  }, [graph]); 

  const formattedGraphData = useMemo(() => {
    if (!graph) return { nodes: [], links: [] };
    
    return getGraphData(graph);
  }, [graph]);

  const isValidClick = () => {
    return !loading
  };

  const handleRecenter = () => {
    if (!isValidClick()) return;
    if (graphRef && graphRef.current) {
      // Set camera's origin, lookAt and up vectors
      graphRef.current.cameraPosition(
        { x: 0, y: 0, z: 40 }, 
        { x: 0, y: 0, z: 0 }               
      );

      graphRef.current.camera().up.set(0, 1, 0);
    }
  };

  const handleOpenFile = async () => {
    if (!isValidClick()) return;
    try {
      let result = await fetchData(setLoading);
      setGraph(result.graph);
      setStats({
        time: result.time, 
        memory: result.memory,
        accuracy: result.accuracy
      })
      setDisabled(result.is_vtk);
    }
    catch (err) {
      console.error(err);
    }
  };

  const handleGraphToggle = async () => {
    if (!isValidClick()) return;

    if (extImg == show_img) {
      await invoke("set_graph_mode", {showTrueGraph: true});
      setExtImg(hide_img);
    }
    else {
      await invoke("set_graph_mode", {showTrueGraph: false});
      setExtImg(show_img);
    }

    try {
      let result = await fetchLastGraph();
      setGraph(result);
    }
    catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="main-container">
      <div className="navbar">
        <button onClick={handleOpenFile} title="Open file">
          <img src={file_open_img} alt="Open File"></img>
        </button>
        <button onClick={handleRecenter} title="Reset camera">
          <img src={reset_img} alt="Recenter"></img>
        </button>
        <button onClick={handleGraphToggle} disabled={disabled} title="Toggle true extremum graph">
          <img src={extImg} alt="Show true extremum graph"></img>
        </button>
      </div>

      {loading && (
        <div className="overlay">
          <div className="spinner" />
        </div>
      )} 

      {!loading && graph && (
        <div className="container" ref={containerRef}>
          {dimensions.width > 0 && dimensions.height > 0 && (
            <ForceGraph3D
              enableNodeDrag={false}
              showNavInfo={false}
              ref={graphRef}
              width={dimensions.width}
              height={dimensions.height}
              graphData={formattedGraphData}
              nodeLabel={node => node.fnVal.toString()}
              nodeColor={node => {if (node.color == 0) return "red"; else return "blue";}}
            />
          )}
          <div className="overlay-text">
            <center>Stats</center>
            Time: {stats.time.toFixed(2)}ms<br/>
            Memory: {fetchMemoryFormattedString(stats.memory)}<br/>
            Accuracy: {stats.accuracy.toFixed(2)}%<br/>
          </div>
        </div>
      )}
    </div>
  );
}