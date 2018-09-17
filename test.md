```pundoc
geometry.margin = "2cm"
citationstyle = "ieee"
```

# Test

## Simple Paragraphs

Paragraph 1, consisting of Line 1
and Line 2.

Paragraph 2

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
![Foobar](100x100.png "placeholdit")
![placeholditimage]

[placeholditimage]: 100x100.png

In the future a biber reference: [@foo].
This a footnote[^foo].

[^foo]: This is fancy text of a footnote

Very large       space.
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

> Single-line block quote

> Multi-line block quote without new tick in the beginning.
This is the second line without indentation.

> Multi-line block quote without new tick in the beginning.
  This is the second line with indentation.

> Multi-line block quote with new tick in the beginning.
> This is the second line.
> **Bold text** *italic*
> -- [@bar]

## Tables

Col 1 | Col 2 | Col 3
:-- | :-: | --:
Left | Center | Right
**Foo** | *Bar* | `Baz`

# Includes

!!include{test-include.md}

* List to include into
  !!include{test-include.md}

1. Numbered list to include into (test continuous state)
  !!include{test-include.md}

[include "foo.md"]

