## Graphviz

See the generated graphviz output in [#graphviz];

```graphviz, #graphviz, caption="Fancy Graph", width=0.5\textwidth, height=0.5\textwidth,scale=0.4
digraph FancyGraph {
    splines=ortho;
    
    start[label="Start Project"];
    right_or_fast[label="Do things\nright or do\nthem fast?", shape="diamond"];
    well[label="Code Well"]
    done_well[label="Are you\ndone yet?", shape="diamond"];
    away[label="Throw it all out\nand start over"];
    fast[label="Code Fast"];
    done_fast[label="Does it\nwork yet?", shape="diamond"];
    
    start -> right_or_fast;
    right_or_fast -> well[label="Right"];
    well -> done_well;
    done_well -> well[label="No"];
    done_well -> away[label="No, and the\nrequirements\nhave changed."];
    right_or_fast -> fast[label="Fast"];
    fast -> done_fast;
    done_fast -> fast[label="No"];
    done_fast -> away[label="Almost, but it's\nbecome a mass\nof kludges and\nspaghetti code."];
    away -> start;
}
```
