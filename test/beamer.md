```pundoc
document_type = "beamer"
lang = "en"
title = "Test Markdown File"
subtitle = "Showing pundoc's Features"
author = "Me"
date = "\\today"
publisher = "My Publisher"
advisor = "My Advisor"
supervisor = "My Supervisor"
citestyle = "ieee"
```

# A very in-depth talk

## Code Formatting

### Rust

Some rust code on a slide:

```rust
fn main() {
    let foo = bar();
}
```

### Graphviz

```graphviz,label=graphviz,caption=Fancy Graph,width=0.5\textwidth,height=0.5\textwidth,scale=0.4
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

# Includes

![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/48/Markdown-mark.svg/208px-Markdown-mark.svg.png)
> -- [@wikipedia-markdown]

[appendix]

[listoflistings]

[listoftables]

[listoffigures]

[bibliography]
