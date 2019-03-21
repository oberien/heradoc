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

[include examples/functionality/tableofcontents.md]

[include examples/functionality/paragraphs.md]

[include examples/functionality/codeblocks.md]

[include examples/functionality/references.md]

[include examples/functionality/images.md]

[include examples/functionality/lists.md]
    
[include examples/functionality/hrule.md]

[include examples/functionality/quotes.md]

[include examples/functionality/tables.md]

[include examples/functionality/includes.md]
  
[include examples/functionality/unicode.md]

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

[include examples/functionality/inline-latex.md]

[appendix]

[include examples/functionality/listoflistings.md]

[include examples/functionality/listoftables.md]

[include examples/functionality/listoffigures.md]

[include examples/functionality/bibliography.md]
