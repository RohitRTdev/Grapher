import { useState, useRef, useEffect, useMemo } from "react";
import type {Graph} from './Types.tsx'
import { getGraphData, fetchData } from "./UI.tsx";
import file_open_img from './assets/file-open.png';
import reset_img from './assets/reset.png';
import ForceGraph3D from "react-force-graph-3d";

export default function App() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  
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

  const handleRecenter = () => {
    if (graphRef && graphRef.current) {
      // Set camera's origin, lookAt and up vectors
      graphRef.current.cameraPosition(
        { x: 0, y: 0, z: 400 }, 
        { x: 0, y: 0, z: 0 }               
      );

      graphRef.current.camera().up.set(0, 1, 0);
    }
  };

  const handleOpenFile = async () => {
    try {
      let result = await fetchData();
      setGraph(result);
    }
    catch (err) {
      console.error(err);
    }
  };

  // Center the graph in the scene once loaded
  useEffect(() => {
    if (!graph) return;

    handleRecenter();
  }, [graph, dimensions]);

  return (
    <div style={{ 
      display: "flex", 
      flexDirection: "column",
      height: "100vh",
      width: "100vw", 
      overflow: "hidden"
    }}>
      <div className="navbar">
        <button onClick={handleOpenFile}>
          <img src={file_open_img} alt="Open File"></img>
        </button>
        <button onClick={handleRecenter}>
          <img src={reset_img} alt="Recenter"></img>
        </button>
      </div>

      {graph && (
        <div className="container" ref={containerRef}>
          {dimensions.width > 0 && dimensions.height > 0 && (
            <ForceGraph3D
              enableNodeDrag={false}
              showNavInfo={false}
              ref={graphRef}
              width={dimensions.width}
              height={dimensions.height}
              graphData={formattedGraphData}
              nodeColor={node => node.color}
              nodeLabel={node => node.name}
            />
          )}
          <div className="overlay-text">Maximum graph</div>
        </div>
      )}
    </div>
  );
}