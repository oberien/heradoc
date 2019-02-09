```pundoc
document_type = "article"
lang = "en"
titlepage = false
title = "Test Markdown File"
subtitle = "Showing pundoc's Features"
author = "Me"
date = "\\today"
publisher = "My Publisher"
advisor = "My Advisor"
supervisor = "My Supervisor"
citestyle = "ieee"
geometry.margin = "2cm"
```

[TOC]

# Test

## Simple Paragraphs

Paragraph 1, consisting of Line 1
and Line 2, but without a hard break.

Paragraph 2 with a very large       space between words which shouldn't be
in the output.

Let's test a linebreak with some lipsum.
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Duis diam velit, dictum ut est nec, hendrerit consequat quam.
Duis aliquam tortor lacus, in ornare orci commodo nec.
Cras vitae lacinia sapien, at scelerisque risus.  
Vestibulum vitae gravida diam.
Aenean maximus nisl vitae egestas euismod.
Vestibulum vitae tortor quis enim dictum dignissim.
Praesent tempor iaculis nunc eu lobortis.

Morbi tincidunt dui nunc, vel egestas urna tempus in.
Etiam a malesuada velit.
Duis sapien metus, ornare at est vel, lacinia elementum metus.
In tincidunt semper nulla.
Nullam pulvinar dolor venenatis metus placerat ornare.
Fusce sed malesuada nibh.
Donec maximus at erat placerat ullamcorper.
Nullam nulla est, vestibulum non consectetur vitae, fringilla a nibh.


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

See the generated graphviz output in [fig:graphviz];

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
Link to existing reference without title with text [Text][Existing-no-title]  
Link to existing reference with title with text [Text][Existing]  
Link to [#Fun-with-references].  
Link to [a section with custom tex][#fun-with-references].  
Inline Link to [a section with custom tex](#fun-with-references).  

[existing-no-title]: Link
[eXisting]: Link "Title"

This is hopefully an image with a title which is used as label, such that I can reference it here as [img:placeholdit].
![Foobar](https://placehold.it/100x100.png "placeholdit")
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

## Lists

* Item 1
* Item 2
    + SubItem 2.1
    + SubItem 2.2
* Item 3

0. Numbered 0
1. Numbered 1
    7. SubNumbered 1.7
    1. SubNumbered 1.8
2. Numbered 2
    1. SubNumbered 2.1
    1. SubNumbered 2.2
    
## Rule (the world)

Text before rule

---
Text after rule

## Block Quote

> "Single-line block quote"

> "Multi-line block quote without new tick in the beginning.
This is the second line without indentation."

> "Multi-line block quote without new tick in the beginning.
  This is the second line with indentation."

> "Multi-line block quote with new tick in the beginning.
> This is the second line.  
> **Bold text** *italic* in separate line.
>
> Here is a new paragraph in the quote."
> -- [@bar]

## Tables

Col 1 | Col 2 | Col 3
:-- | :-: | --:
Left | Center | Right
**Foo** | *Bar* | `Baz`

# Includes

[include test-include.md]

* List to include into
  [include test-include.md]

1. Numbered list to include into (test continuous state)
  ![][test-include.md]
  
> From web:
> [include https://raw.githubusercontent.com/oberien/pundoc/master/test-include.md]
> -- [@foo]

![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/48/Markdown-mark.svg/208px-Markdown-mark.svg.png)
  
# Unicode Substitution

A → B ⇒ C ≤ D

In math: `$ A ← B ⇐ C ∈ D ⊥ E`

Multiline substitution ⊂∫∀∃ΞΛℂ∈ΣℕℝΔ  
Check for linespacing EEEEEEEEEEEEEEE

Text with_underscore and `inline code with_underscore` and `$ math\ with_{subscript}` and `# symbols` and even \\# outside any other context.
```
code block with_underscore
# And a bash-style comment with #
```

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
