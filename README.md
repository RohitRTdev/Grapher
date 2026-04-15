# Grapher
A simple UI tool to extract data from scalar fields and display the corresponding extremum graph. The goal is not just to build it, but 
to use a <b>streaming algorithm</b> as described in this paper [Scalable topological data analysis](https://arxiv.org/abs/1907.08325). 
The algorithm is based on techniques described in paper [Efficient computation of extremum graphs](https://arxiv.org/abs/2303.02724). 
The streaming algorithm focuses on reducing the large memory footprint often associated with computation of extremum graph for very high dimensional manifolds.

# Build
To do development/debug build
```
npm run tauri dev
```

To do release build 
```
npm run tauri build
```

## Dependencies
* node
* npm