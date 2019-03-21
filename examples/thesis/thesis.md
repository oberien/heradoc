```config
lang = "en"
document_type = "thesis"
title = "Fancy Title"
subtitle = "Subtitle (e.g. title in native language)"
author = "My Name"
date = "\\today"
supervisor = "My Supervisor"
advisor = "My Advisor"
logo_university = "https://cdn.pixabay.com/photo/2017/12/17/14/12/bitcoin-3024279_960_720.jpg"
logo_faculty = "https://en.bitcoin.it/w/images/en/2/29/BC_Logo_.png"
university = "Univeristy of Duckburg"
faculty = "Department of Quak"
thesis_type = "Bachelor's Thesis in Quak"
location = "Duckburg"
disclaimer = "I confirm that this bachelor's thesis is my own work and I have documented all sources and material used."
abstract = "abstract.md"
abstract2 = "abstract2.md"
bibliography = "references.bib"
header_includes = ["\\usepackage{lipsum}"]
geometry.papersize = "a4"
```

# Section 1

Let's start with citing [@foo].
Then we'll make a table, which we can reference as [#tbl:table]

{#tbl:table, caption="Fancy Table"}

Header 1 | Header 2
:-- | :-:
Cell 1 | Cell 2
Cell 3 | Cell 4

Now we'll also show listing [#lst:listing]

{#lst:listing, caption="Hello World in Brainfuck"}
```
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>
---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

\lipsum[1-3]

## Subsection 1.1

\lipsum[4-10]

### Subsubsection 1.1.1

\lipsum[11-15]

## Subsection 1.2

\lipsum[16-20]

# Section 2

\lipsum[21-30]

[appendix]

[listoffigures]

[listoftables]

[listoflistings]

[bibliography]
