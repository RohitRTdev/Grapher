import { Graph } from "./Types.ts";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

export function getGraphData(graph: Graph) {
    return {
      nodes: graph.nodes.map(e => ({
        id: e.id,
        color: "blue",
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

export async function fetchData(loadCallback: (start: boolean) => void) : Promise<Graph> {
    const selected = await open({
        multiple: false,
        filters: [
        { name: "VTK Files", extensions: ["vtk"] }
        ]
    });

    if (!selected || Array.isArray(selected)) {
        throw new Error("Multiple selection / no selection applied!");
    }
    
    loadCallback(true);        

    try {
        const result = await invoke<Graph>("process_vtk_file_async", {
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