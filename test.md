```heradoc
document_type = "article"
lang = "en"
titlepage = false
title = "Test Markdown File"
subtitle = "Showing heradoc's Features"
author = "Me"
date = "\\today"
publisher = "My Publisher"
advisor = "My Advisor"
supervisor = "My Supervisor"
citestyle = "ieee"
geometry.margin = "2cm"
```

[TOC]

[include examples/functionality/paragraphs.md]

## Code Formatting

### Rust

Some rust code:

```rust
fn main() {
    let foo = bar();
}
```

### Sequence Diagram?

In the future possibly a sequence diagram.

```sequence
uiae
```

### Graphviz

See the generated graphviz output in [#graphviz];

```graphviz,#graphviz,caption=Fancy Graph,width=0.5\textwidth,height=0.5\textwidth,scale=0.4
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

### Python

At least we already have python:

```python
def foo():
    if True:
        print(1337)
foo()
```

[include examples/functionality/references.md]

[include examples/functionality/images.md]

[include examples/functionality/lists.md]
    
[include examples/functionality/hrule.md]

[include examples/functionality/quotes.md]

[include examples/functionality/tables.md]

[include examples/functionality/includes.md]
  
# Unicode Substitution

A → B ⇒ C ≤ D

In math: `$ A ← B ⇐ C ∈ D ⊥ E`

Multiline substitution ⊂∫∀∃ΞΛℂ∈ΣℕℝΔ  
Check for linespacing EEEEEEEEEEEEEEE

Text with # sharp.
Text with \\{} backslash curly parentheses.
Text with_underscore and `inline code with_underscore` and `$ math\ with_{subscript}`.
```
code block with_underscore
```
Inline code `with \ backslash`. Inline code `with # sharp`.

# Math

Inline math `$ \forall x \in \mathbb{N} : \exists y \in \mathbb{N} : y > x`.

Equation without number:

```$$
\sqrt{-i}\quad 2^3&\quad \sum\quad \pi\\
\ldots\; and\; it&\; was\; delicious
```

Equation with number:

```$$$
a &= b\\
a^2 &= ab\\
a^2 - b^2 &= ab - b^2\\
(a + b) (a - b) &= b(a - b)\\
a + b &= b\\
b + b &= b\\
2b &= b\\
2 &= 1
```

[appendix]

[include examples/functionality/listoflistings.md]

[include examples/functionality/listoftables.md]

[include examples/functionality/listoffigures.md]

[include examples/functionality/bibliography.md]
