Pundoc is a markdown to LaTeX converter.
It is a partial reimplementation of [pandoc](https://pandoc.org/MANUAL.html) with a pinch of
[hackmd](https://hackmd.io/features?both).

# Features

- [x] includes of other md files (`!!include{foo.md}`)
- [x] cite author formatting (`> -- [@foo]`)
- [x] renders ok-ish in commonmark renderers
- [x] produces readable latex
    + worst case: fall back to latex if pundoc fails
- [x] generate links for sections (non-alphanumerics replaced with `-`, all lowercase)
- [x] Footnotes (currently the footnote is on the page where the footnote definition is placed, not its first reference)
- [ ] hrule (currently buggy indentation after)
- [ ] tables: merge columns
- [ ] tables: merge rows
- [ ] tables: merge columns and rows (e.g. 3x3 field)
- [ ] comments
- [ ] ```` ```sequence````
- [ ] ```` ```graphviz````
- [ ] ```` ```flow````
- [ ] ```` ```gnuplot````
- [ ] ```` ```mermaid````
- [ ] ```` ```abc````
- [ ] biber support: `[@foo]` references biber (maybe not if HTML backend?)
- [ ] `[TOC]`
- [ ] code-blocks with inline unicode / math-mode (`\begin{lstlisting}[mathescape=true]`)
- [ ] unicode support (for common symbols, translate into latex math equivalents, e.g. →, basically neo layer 6 :D )
    - [ ] typographic replacements (e.g. `(c)`, `(r)`, `(tm)`)
    - [ ] auto-detect unicode in listing and enable mathescape
- [ ] inline latex
    + still render markdown between `\begin` and `\end` etc, which pandoc doesn't
    + not if HTML as backend
- [ ] citation style (.cs)
- [ ] todolist (enumitem)
- [ ] includes of files other than images / md
- [ ] inline latex math mode (`$foo$`)
- [ ] equation without number (`$$\nfoo\n$$`)
- [ ] equation with number (`$$$\nfoo\n$$$`)
- [ ] alert area??? (success, info, warning, danger)
- [ ] superscript (`foo^bar^`)
- [ ] subscript (`foo~bar~`)
- [ ] image size

# Config Options

- [ ] output (file / stdout)
- [ ] out-type (tex, pdf, …)
- [ ] pdf metadata (examine which)
- [ ] papersize
- [ ] documentclass
- [ ] geometry
- [ ] itemizespacing
- [ ] use minted instead of lstlistings
- [ ] lstset
- [ ] graphicspath
- [ ] cleveref options
- [ ] let footnotes appear where they are first used vs where they are declared
- [ ] make softbreaks (line breaks) hard brakes (line ends with 2 spaces)
- [ ] header includes

# Cli

- [x] `pundoc -o bar.pdf bar.md`
- [ ] configuration file (allows `pundoc build`)?
- [ ] configuration directly in .md file like pandoc, but in better
    - pandoc header renders bad in other markdown renderers
    - ```` ```header```` or ```` ```preamble````
- [ ] `pundoc bar.md` (short for `pundoc -o bar.pdf bar.md`)?
- [ ] every cli option must be configurable in the header (except `-o` and similar?; maybe `-o` allowed but only in subfolders)
- [ ] cli overrides header overrides config-file overrides defaults

# Backend

- latex backend
    - [x] scrartcl
    - [ ] beamer
- HTML backend
    + get rid of latex altogether
    + [ ] book
    + [ ] slides
- [ ] Generation via file templates
    + separate templates for headers and body
    + one template which headers and body get rendered into
    + http://www.arewewebyet.org/topics/templating/

# Frontend:

- [x] pulldown-cmark
