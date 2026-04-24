import { Graph, FinalResult } from "./Types.ts";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

export function getGraphData(graph: Graph) {
    console.log(graph);
    return {
      nodes: graph.nodes.map(e => ({
        id: e.id,
        color: e.color_code,
        fnVal: e.fn_val,
        fx: e.x * 100, 
        fy: e.y * 100,
        fz: e.z * 100
      })),
      links: graph.edges.map(e => ({
        source: e.source,
        target: e.target
      }))
    }; 
}

export async function fetchData(loadCallback: (start: boolean) => void) : Promise<FinalResult> {
    const selected = await open({
        multiple: false,
        filters: [
        { name: "VTK/Manifold Files", extensions: ["vtk", "man"] }
        ]
    });

    if (!selected || Array.isArray(selected)) {
        throw new Error("Multiple selection / no selection applied!");
    }
    
    loadCallback(true);        

    try {
        const result = await invoke<FinalResult>("process_file_async", {
        path: selected
        });

        return result;
    } catch (err) {
        console.error(err);
    } finally {
        loadCallback(false);
    }

    throw new Error("Failed to fetch graph data!");
}