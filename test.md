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

## Fun with References

Link to non-existent reference without text [Nonexistent].  
Link to non-existent reference with text [Text][Nonexistent].  
Inline link without title [Text](Link).  
Inline link with title [Text](Link "Title").  
Link to existing reference without title without text [Existing-no-title].  
Link to existing reference with title without text [Existing].  
Link to existing reference without title with text [Text][Existing-no-title].  
Link to existing reference with title with text [Text][Existing].  
Capital link to [#Fun-with-references].  
Small link to [#fun-with-references].  
Link to [a section with custom tex][#fun-with-references].  
Inline Link to [a section with custom tex](#fun-with-references).  
A collapsed link to a section [#fun-with-references][].

[existing-no-title]: Link
[eXisting]: Link "Title"

This is hopefully an image with a title which is used as label, such that I can reference it here as [#placeholdit].

{#placeholdit,nofigure, caption = "Caption with \\"quotes\\" and , comma"}
![This is a fancy alt-text](https://placehold.it/100x100.png "placeholdit image")
![placeholditimage]

[placeholditimage]: https://placehold.it/100x100.png

A biber reference: [@foo].  
A biber reference with page: [@foo 1337].  
A biber reference with multiple pages: [@foo 42-69, 112].  
A biber reference with chapter: [@foo Chapter 13].  
Multiple collapsed biber references: [@foo, @bar].  
Multiple collapsed biber references with pages: [@foo 21-23,25, @bar Chapters 10-15].  
This a footnote[^foo].

[^foo]: This is fancy text of a footnote

[include examples/functionality/lists.md]
    
[include examples/functionality/hrule.md]

[include examples/functionality/quotes.md]

[include examples/functionality/tables.md]

# Includes

[include test-include.md]

* List to include into
  [include test-include.md]

1. Numbered list to include into (test continuous state)
  ![](test-include.md)
  
> From web:
> [include https://raw.githubusercontent.com/oberien/heradoc/master/test-include.md]
> -- [@foo]

![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/48/Markdown-mark.svg/208px-Markdown-mark.svg.png)

[include https://placehold.it/100x100.png]
  
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

[listoflistings]

[listoftables]

[listoffigures]

[bibliography]
