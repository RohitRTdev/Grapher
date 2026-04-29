import { Graph, FinalResult } from "./Types.ts";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

export function getGraphData(graph: Graph) {
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

        console.log(result);
        return result;
    } catch (err) {
        console.error(err);
    } finally {
        loadCallback(false);
    }

    throw new Error("Failed to fetch graph data!");
}

export async function fetchLastGraph() : Promise<Graph> {
    try {
        const result = await invoke<Graph>("retrieve_last_graph");
        return result;
    } catch (err) {
        console.error(err);
    } 

    throw new Error("Failed to fetch previous graph data!");
}

export function fetchMemoryFormattedString(memory: number) : string {
    let divFactor = 1;
    let suffix = "B";

    if (memory >= 1024 * 1024 * 1024) {
        divFactor = 1024 * 1024 * 1024;
        suffix = "GiB";
    } 
    else if (memory >= 1024 * 1024) {
        divFactor = 1024 * 1024;
        suffix = "MiB";
    }
    else if (memory >= 1024) {
        divFactor = 1024;
        suffix = "KiB";
    }
    else {
        return memory + suffix;
    }

    return (memory / divFactor).toFixed(2) + suffix
}

export function fetchTimeFormattedString(time: number) : string {
    let newTime = time;
    let suffix = "ms";
    
    if (time >= 60 * 1000) {
        newTime = time / 1000 / 60;
        suffix = "min";
    }
    else if (time >= 1000) {
        newTime = time / 1000;
        suffix = "s"; 
    }

    return newTime.toFixed(2) + suffix;
}