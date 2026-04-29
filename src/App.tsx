import { useState, useRef, useEffect, useMemo } from "react";
import type {Graph, Stats, FinalResult} from './Types.ts'
import { invoke } from "@tauri-apps/api/core";
import { getGraphData, fetchData, fetchMemoryFormattedString, fetchTimeFormattedString, fetchLastGraph } from "./UI.ts";
import file_open_img from './assets/file-open.png';
import reset_img from './assets/reset.png';
import show_img from './assets/show.png';
import hide_img from './assets/hide.png';
import plus_img from './assets/plus.png';
import minus_img from './assets/minus.png';
import up_img from './assets/up.png';
import down_img from './assets/down.png';
import max_img from './assets/max.png';
import min_img from './assets/min.png';
import ForceGraph3D from "react-force-graph-3d";

export default function App() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState<Stats>({
                              time: 0,
                              memory: 0,
                              accuracy: 0,
                              f1score: 0
                            });

  
  const [disabled, setDisabled] = useState<boolean>(true);
  const [extImg, setExtImg] = useState<string>(show_img);
  const [K, setK] = useState<number>(10);
  const [beta, setBeta] = useState<number>(0.1);
  const [isMaximum, setIsMaximum] = useState<boolean>(true);
  const [maxDisabled, setMaxDisabled] = useState<boolean>(true);
  const graphRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const stepK = 50;
  const stepBeta = 0.1;
  
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
        accuracy: result.accuracy,
        f1score: result.f1score
      })
      setDisabled(result.is_vtk);
      setMaxDisabled(false);
    }
    catch (err) {
      console.error(err);
    }
  };

  const handleGraphToggle = async () => {
    if (!isValidClick()) return;

    if (extImg == show_img) {
      await invoke("set_graph_mode", {showTrueGraph: true, isMaxGraph: isMaximum});
      setExtImg(hide_img);
    }
    else {
      await invoke("set_graph_mode", {showTrueGraph: false, isMaxGraph: isMaximum});
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

  const handleIncDecCommon = async (new_k: number, new_beta: number) => {
    try {
      await invoke("set_config", {k: new_k, beta: new_beta});
      setLoading(true);
      let result = await invoke<FinalResult>("recompute_graph");
      setLoading(false);

      console.log(result);
      setGraph(result.graph);
      setStats({
        time: result.time, 
        memory: result.memory,
        accuracy: result.accuracy,
        f1score: result.f1score
      })
    }
    catch (err) {
      console.error(err);
    }
  };

  const handleIncrement = async () => {
    if (!isValidClick) return;
    if (K < 500) {
      await handleIncDecCommon(K + stepK, beta);
      setK(K + stepK);
    }
  };

  const handleDecrement = async () => {
    if (!isValidClick) return;
    if (K > 10) {
      await handleIncDecCommon(K - stepK, beta);
      setK(K - stepK);
    }
  };

  const handleBetaIncrement = async () => {
    if (!isValidClick) return;
    if (beta < 1) {
      await handleIncDecCommon(K, beta + stepBeta);
      setBeta(beta + stepBeta);
    }
  };

  const handleBetaDecrement = async () => {
    if (!isValidClick) return;
    if (beta > 0.1) {
      await handleIncDecCommon(K, beta - stepBeta);
      setBeta(beta - stepBeta);
    }
  };

  // Switch to max/min graph
  const handleGraphMode = async () => {
    if (!isValidClick) return;
    try {
      await invoke("set_graph_mode", {showTrueGraph: extImg == hide_img, isMaxGraph: !isMaximum})
      await handleIncDecCommon(K, beta); 
      setIsMaximum(!isMaximum);
    }
    catch (err) {
      console.error(err);
    }
  }

  return (
    <div className="main-container">
      <div className="navbar">
        <button onClick={handleOpenFile} title="Open file">
          <img src={file_open_img} alt="Open File"></img>
        </button>
        <button onClick={handleRecenter} title="Reset camera">
          <img src={reset_img} alt="Recenter"></img>
        </button>
        <button onClick={handleGraphMode} disabled={maxDisabled} title="Toggle max/min graph">
          <img src={isMaximum ? max_img : min_img} alt="Toggle max/min mode"></img>
        </button>
        <button onClick={handleGraphToggle} disabled={disabled} title="Toggle true extremum graph">
          <img src={extImg} alt="Show true extremum graph"></img>
        </button>
        <button onClick={handleIncrement} disabled={disabled || K >= 500} title="Increment K">
          <img src={plus_img}></img>
        </button>
        <button onClick={handleDecrement} disabled={disabled || K <= 10} title="Decrement K">
          <img src={minus_img}></img>
        </button>
        <button onClick={handleBetaIncrement} disabled={disabled || beta.toFixed(2).startsWith("1")} title="Increment beta">
          <img src={up_img}></img>
        </button>
        <button onClick={handleBetaDecrement} disabled={disabled || beta.toString().startsWith("0.1")} title="Decrement beta">
          <img src={down_img}></img>
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
              nodeColor={node => {if (node.color == 0) return (isMaximum ? "red" : "green") ; else return "blue";}}
            />
          )}
          <div className="overlay-text">
            <center>Stats</center>
            Graph type: {isMaximum ? "Maximum" : "Minimum"}<br/>
            KNN: {K}<br/>
            Beta: {beta.toFixed(1)} (Using refined ERG)<br/>
            Time: {fetchTimeFormattedString(stats.time)}<br/>
            Memory: {fetchMemoryFormattedString(stats.memory)}<br/>
            Accuracy: {stats.accuracy.toFixed(2)}%<br/>
            F1 score: {stats.f1score.toFixed(2)}%<br/>
          </div>
        </div>
      )}
    </div>
  );
}