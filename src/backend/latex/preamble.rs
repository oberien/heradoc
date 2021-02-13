use std::io::{Result, Write};

use isolang::Language;

use crate::config::{Config, DocumentType};
use crate::util::{OutJoiner, ToUnix};
use crate::diagnostics::Diagnostics;

/// Writes the documentclass header of latex documents with the given class and options.
///
/// Also adds options specified in the config like the fontsize, titlepage and manual classoptions.
pub fn write_documentclass(cfg: &Config, out: &mut impl Write, class: &str, options: &str) -> Result<()> {
    write!(out, "\\documentclass[")?;
    write!(out, "{},", cfg.fontsize)?;
    write!(out, "{}", options)?;
    match cfg.titlepage {
        true => write!(out, "titlepage,")?,
        false => write!(out, "notitlepage,")?,
    }
    for other in &cfg.classoptions {
        write!(out, "{},", other)?;
    }
    writeln!(out, "]{{{}}}", class)?;
    writeln!(out)?;
    Ok(())
}

/// Writes required usepackage statements.
pub fn write_packages(cfg: &Config, out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\usepackage[utf8]{{inputenc}}")?;
    writeln!(out, "\\usepackage[T1]{{fontenc}}")?;
    writeln!(out, "\\usepackage[sc]{{mathpazo}}")?;
    let lang = match cfg.lang {
        Language::Deu => "ngerman".to_string(),
        lang => lang.to_name().to_ascii_lowercase(),
    };
    writeln!(out, "\\usepackage[{}]{{babel}}", lang)?;
    writeln!(out, "\\usepackage{{csquotes}}")?;
    writeln!(out, "\\MakeOuterQuote{{\"}}")?;

    // geometry
    write!(out, "\\usepackage[")?;
    cfg.geometry.write_latex_options(&mut *out)?;
    writeln!(out, "]{{geometry}}")?;

    writeln!(out)?;

    // TODO: biblatex options (natbib?)
    if let Some(bibliography) = &cfg.bibliography {
        writeln!(
            out,
            "\\usepackage[backend=biber,citestyle={},bibstyle={}]{{biblatex}}",
            cfg.citestyle, cfg.bibstyle
        )?;
        writeln!(out, "\\addbibresource{{{}}}", bibliography.to_unix()
            .expect(&format!("non-utf8 path: {:?}", bibliography)))?;
    }

    writeln!(out, "\\usepackage{{cmap}}")?;
    writeln!(out, "\\usepackage{{float}}")?;
    // TODO: use minted instead of lstlistings?
    // TODO: do we want scrhack?
    writeln!(out, "\\usepackage{{listings}}")?;
    writeln!(out, "\\usepackage[usenames, dvipsnames]{{color}}")?;
    writeln!(out, "\\usepackage{{xcolor}}")?;
    writeln!(out, "\\usepackage{{pdfpages}}")?;
    writeln!(out, "\\usepackage{{environ}}")?;
    writeln!(out, "\\usepackage{{amssymb}}")?;
    writeln!(out, "\\usepackage{{amsmath}}")?;
    writeln!(out, "\\usepackage{{stmaryrd}}")?;
    writeln!(out, "\\usepackage[gen]{{eurosym}}")?;
    writeln!(out, "\\usepackage[normalem]{{ulem}}")?;
    // TODO: graphicspath (probably not needed due to our own file resolution system (resolve))
    writeln!(out, "\\usepackage{{graphicx}}")?;
    writeln!(out, "\\usepackage{{transparent}}")?;
    writeln!(out, "\\usepackage[final]{{microtype}}")?;
    writeln!(out, "\\usepackage[pdfusetitle]{{hyperref}}")?;
    writeln!(out, "\\usepackage{{caption}}")?;
    // TODO: cleveref options
    writeln!(out, "\\usepackage[{}]{{cleveref}}", lang)?;
    writeln!(out, "\\usepackage{{refcount}}")?;
    writeln!(out, "\\usepackage[titletoc,toc,title]{{appendix}}")?;
    writeln!(out, "\\usepackage{{array}}")?;
    writeln!(out, "\\usepackage{{pdfcomment}}")?;
    writeln!(out, "\\usepackage{{tabularx}}")?;
    // auto-wrap long tables at page boundaries
    writeln!(out, "\\usepackage{{ltablex}}")?;
    writeln!(out, "\\usepackage{{grffile}}")?;
    let fancy = [
        ("lhead", &cfg.lhead, &cfg.lhead_even), ("chead", &cfg.chead, &cfg.chead_even), ("rhead", &cfg.rhead, &cfg.rhead_even),
        ("lfoot", &cfg.lfoot, &cfg.lfoot_even), ("cfoot", &cfg.cfoot, &cfg.cfoot_even), ("rfoot", &cfg.rfoot, &cfg.rfoot_even),
    ];
    let mut fancyhdr = Vec::new();
    for (name, odd, even) in &fancy {
        if odd.is_none() && even.is_none() {
            continue;
        }
        write!(fancyhdr, "\\{}", name)?;
        if let Some(even) = even {
            write!(fancyhdr, "[{}]", even)?;
        }
        writeln!(fancyhdr, "{{{}}}", odd.as_ref().map(String::as_ref).unwrap_or(""))?;
    }
    if !fancyhdr.is_empty() {
        writeln!(out, "\\usepackage{{fancyhdr}}")?;
        writeln!(out, "\\pagestyle{{fancy}}")?;
        writeln!(out, "\\fancyhf{{}}")?;
        write!(out, "{}", String::from_utf8(fancyhdr).expect("non-utf8 fancyhdr configuration string"))?;
        if cfg.header_footer_on_titlepage {
            // writeln!(out, "\\pagestyle{{fancyplain}}")?;
            // writeln!(out, "\\makeatletter\\let\\ps@plain\\ps@fancy\\makeatother")?;
            writeln!(out, "\\fancypagestyle{{plain}}{{\\pagestyle{{fancy}}}}")?;
            writeln!(out, "\\fancypagestyle{{empty}}{{\\pagestyle{{fancy}}}}")?;
        }
    }
    writeln!(out)?;
    Ok(())
}

/// Writes customized fixes of commands, configuration options of packages, custom commands
/// and similar, which are used by latex code generation.
pub fn write_fixes(cfg: &Config, out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\setlength{{\\parindent}}{{0pt}}")?;
    writeln!(out, "\\setlength{{\\parskip}}{{1\\baselineskip plus 2pt minus 2pt}}")?;
    writeln!(out, "{}", LSTSET)?;
    writeln!(out, "{}", LST_DEFINE_ASM)?;
    writeln!(out, "{}", LST_DEFINE_RUST)?;
    writeln!(out, "{}", LST_DEFINE_JS)?;
    writeln!(out, "{}", THICKHLINE)?;
    writeln!(out, "{}", AQUOTE)?;
    writeln!(out, "{}", FIX_INCLUDEGRAPHICS)?;
    writeln!(out, "{}", IMAGE_WITH_TEXT)?;
    writeln!(out, "{}", SCALE_TIKZ_PICTURE_TO_WIDTH)?;
    writeln!(out, "{}", TABULARX)?;
    // TODO: figures inline? https://tex.stackexchange.com/a/11342 last codeblock
    // with package float and `[H]`

    for include in &cfg.header_includes {
        writeln!(out, "{}", include)?;
    }
    Ok(())
}

pub fn write_deep_headers(cfg: &Config, out: &mut impl Write) -> Result<()> {
    fn write_simple_header(name: &str, into: &mut impl Write,  font: &str, level: i32) -> Result<()> {
        writeln!(into, "{}", "\\makeatletter")?;
        write!(into, r#"\newcommand\{}"#, name)?;
        write!(into, "{}{}{}{}", r#"{\@startsection{"#, name, "}{", level)?;
        write!(into, "{}", r#"}{\z@}%
                                {-3.5ex \@plus -1ex \@minus -.2ex}%
                                {2.3ex \@plus.2ex}%
                                {\sffamily"#)?;
        write!(into, "{}", font)?;
        write!(into, "{}", r#"}
            }
            \newcommand{"#)?;
        write!(into, r#"\{}mark"#, name)?;
        writeln!(into, "{}", r#"}[1]{}"#)?;
        writeln!(into, "{}", "\\makeatother")?;
        Ok(())
    }

    write_simple_header("heradocsectionfour", out, "\\fontsize{14.4pt}{\\f@baselineskip}\\fontseries{b}\\selectfont", 4)?;
    write_simple_header("heradocsectionfive", out, "\\fontsize{12pt}{\\f@baselineskip}\\fontseries{b}\\selectfont", 5)?;
    write_simple_header("heradocsectionsix", out, "\\fontsize{12pt}{\\f@baselineskip}\\fontshape{it}\\selectfont", 6)?;
    Ok(())
}

/// Allows configuration of the short author field when writing the maketitle info.
///
/// Some document types like beamer support a short author field like `\author[short-author]{author}`.
/// However, this is not supported in standard KOMA, so it should only be used when supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortAuthor {
    /// Include the short author field: `\author[short-author]{author}`
    Yes,
    /// Don't include the short author field
    No,
}

/// Sets up variables for the latex maketitle titlepage.
///
/// It doesn't write the titlepage itself, that is left to the caller.
pub fn write_maketitle_info(cfg: &Config, short_author: ShortAuthor, out: &mut impl Write, diagnostics: &Diagnostics<'_>) -> Result<()> {
    if let Some(title) = &cfg.title {
        writeln!(out, "\\title{{{}}}", title)?;
    }
    if let Some(subtitle) = &cfg.subtitle {
        writeln!(out, "\\subtitle{{{}}}", subtitle)?;
    }
    if cfg.titlehead.is_some() || cfg.logo_university.is_some() {
        match cfg.document_type {
            DocumentType::Article | DocumentType::Report | DocumentType::Thesis | DocumentType::RustDoc => {
                writeln!(out, "\\titlehead{{")?;
                let mut joiner = OutJoiner::new(&mut *out, "\\\\");
                if let Some(titlehead) = &cfg.titlehead {
                    joiner.join(format_args!("\\raggedright {}", titlehead))?;
                }
                if let Some(logo_university) = &cfg.logo_university {
                    joiner.join(format_args!("\\vspace{{5mm}}\\centering\\includegraphics[height=20mm]{{{}}}", logo_university.to_unix().expect("can't convert univerity logo path to unix")))?;
                }
                writeln!(out, "}}")?;
            }
            DocumentType::Beamer => diagnostics
                .warning("the titlehead config property is not supported by beamer")
                .emit(),
        }
    }
    if cfg.author.is_some() || cfg.supervisor.is_some() || cfg.advisor.is_some() {
        write!(out, "\\author")?;
        if short_author == ShortAuthor::Yes {
            if let Some(author) = &cfg.author {
                write!(out, "[{}]", author)?;
            }
        }
        write!(out, "{{")?;
        let mut joiner = OutJoiner::new(&mut *out, "\\\\");
        if let Some(author) = &cfg.author {
            joiner.join(format_args!("{}", author))?;
        }
        if let Some(supervisor) = &cfg.supervisor {
            joiner.join(format_args!("Supervisor: {}", supervisor))?;
        }
        if let Some(advisor) = &cfg.advisor {
            joiner.join(format_args!("Advisor: {}", advisor))?;
        }
        writeln!(out, "}}")?;
    }
    if let Some(date) = &cfg.date {
        writeln!(out, "\\date{{{}}}", date)?;
    }
    if let Some(email) = &cfg.email {
        writeln!(out, "\\email{{{}}}", email)?;
    }
    if cfg.university.is_some() || cfg.faculty.is_some() || cfg.publisher.is_some() || cfg.logo_faculty.is_some() {
        let mut publishers = Vec::new();
        let mut joiner = OutJoiner::new(&mut publishers, "\\\\");
        if let Some(university) = &cfg.university {
            joiner.join(format_args!("{}", university))?;
        }
        if let Some(faculty) = &cfg.faculty {
            joiner.join(format_args!("{}", faculty))?;
        }
        if let Some(publisher) = &cfg.publisher {
            joiner.join(format_args!("{}", publisher))?;
        }
        if let Some(logo_faculty) = &cfg.logo_faculty {
            joiner.join(format_args!("\\vspace{{5mm}}\\includegraphics[height=20mm]{{{}}}", logo_faculty.to_unix().expect("can't convert faculty logo path to unix")))?;
        }
        writeln!(out, "\\publishers{{{}}}", String::from_utf8(publishers).expect("invalid UTF-8 publishers"))?;
    }
    writeln!(out)?;

    Ok(())
}

/// If using a manual titlepage template, like Thesis, this command defines latex commands
/// to be used within that template.
pub fn write_manual_titlepage_commands(cfg: &Config, out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\def \\ifempty#1{{\\ifx\\empty#1}}")?;
    fn get(o: &Option<String>) -> &str {
        o.as_ref().map_or("", |s| s.as_str())
    }
    writeln!(out, "\\newcommand*{{\\getTitle}}{{{}}}", get(&cfg.title))?;
    writeln!(out, "\\newcommand*{{\\getSubtitle}}{{{}}}", get(&cfg.subtitle))?;
    writeln!(out, "\\newcommand*{{\\getTitlehead}}{{{}}}", get(&cfg.titlehead))?;
    writeln!(out, "\\newcommand*{{\\getAuthor}}{{{}}}", get(&cfg.author))?;
    writeln!(out, "\\newcommand*{{\\getEmail}}{{{}}}", get(&cfg.email))?;
    writeln!(out, "\\newcommand*{{\\getDate}}{{{}}}", get(&cfg.date))?;
    writeln!(out, "\\newcommand*{{\\getSupervisor}}{{{}}}", get(&cfg.supervisor))?;
    writeln!(out, "\\newcommand*{{\\getAdvisor}}{{{}}}", get(&cfg.advisor))?;
    if let Some(logo_university) = cfg.logo_university.as_ref() {
        writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{{}}}", logo_university.to_unix()
            .expect(&format!("non-utf8 path: {:?}", logo_university)))?;
    } else {
        writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{}}")?;
    }
    if let Some(logo_faculty) = cfg.logo_faculty.as_ref() {
        writeln!(out, "\\newcommand*{{\\getLogoFaculty}}{{{}}}", logo_faculty.to_unix()
            .expect(&format!("non-utf8 path: {:?}", logo_faculty)))?;
    } else {
        writeln!(out, "\\newcommand*{{\\getLogoFaculty}}{{}}")?;
    }
    writeln!(out, "\\newcommand*{{\\getUniversity}}{{{}}}", get(&cfg.university))?;
    writeln!(out, "\\newcommand*{{\\getFaculty}}{{{}}}", get(&cfg.faculty))?;
    writeln!(out, "\\newcommand*{{\\getThesisType}}{{{}}}", get(&cfg.thesis_type))?;
    writeln!(out, "\\newcommand*{{\\getLocation}}{{{}}}", get(&cfg.location))
}

// https://en.wikibooks.org/wiki/LaTeX/Source_Code_Listings
pub const LSTSET: &str = r#"
\lstset{%
  numbers=left,
  numberstyle=\tiny\color{gray},
  stepnumber=1,
  numbersep=5pt,
  showspaces=false,
  showstringspaces=false,
  showtabs=false,
  frame=single,
  rulecolor=\color{black},
  tabsize=8,
  captionpos=b,
  breaklines=true,
  breakatwhitespace=false,
  language=C,
  keywordstyle=\bfseries\color{OliveGreen},
  commentstyle=\itshape\color{Mahogany},
  stringstyle=\color{BrickRed},
  keywordstyle=[2]{\color{Cyan}},
  xleftmargin=8pt,
  xrightmargin=3pt,
  basicstyle=\scriptsize,
  morekeywords={u32, __u32, __be32, __le32,
  		u16, __u16, __be16, __le16,
	        u8,  __u8,  __be8,  __le8,
          size_t, ssize_t, __int8,
      _BYTE, LOBYTE, BYTE1, BYTE2, BYTE3}
}
"#;

// https://tex.stackexchange.com/questions/51645/
pub const LST_DEFINE_ASM: &str = r#"
%  x86-64-assembler-language-dialect-for-the-listings-package
\lstdefinelanguage
   [x86_64]{Assembler}
   [x86masm]{Assembler}
   % with these extra keywords:
   {morekeywords={CDQE, CQO, CMPSQ, CMPXCHG16B, JRCXZ, LODSQ, MOVSXD,
                  POPFQ, PUSHFQ, SCASQ, STOSQ, IRETQ, RDTSCP, SWAPGS,
                  B, BX, LDR.W, DCD, =,
                  rax, rdx, rcx, rbx, rsi, rdi, rsp, rbp,
                  r8, r8d, r8w, r8b, r9, r9d, r9w, r9b,
                  LR}}
"#;

pub const LST_DEFINE_RUST: &str = r#"
\lstdefinelanguage{rust}{%
  keywords={%
    % strict keywords
    as, break, const, continue, crate, else, enum, extern, false, fn,
    for, if, impl, in, let, loop, match, mod, move, mut, pub, ref,
    return, self, Self, static, struct, super, trait, true, type,
    unsafe, use, where, while,
    % reserved keywords
    abstract, become, box, do, final, macro, override, priv, typeof,
    unsized, virtual, yield,
    % weak keywords
    union, dyn,
  },
  keywords=[2]{%
    i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize,
    f32, f64
  },
  keywords=[3]{%
    'a, 'b, 'static,
    [,],&,
    Some, None, Ok, Err
  },
  morecomment=[s]{/*}{*/},
  morecomment=[l]//,
  morestring=[b]"
  %morestring=[b]'
}[keywords,comments,strings,directives]
"#;

pub const LST_DEFINE_JS: &str = r#"
\lstdefinelanguage{js}{
  keywords={typeof, new, true, false, catch, function, return, null, catch, switch, var, let, const, if, in, while, do, else, case, break},
  keywordstyle=\color{blue}\bfseries,
  ndkeywords={class, export, boolean, throw, implements, import, this, console},
  ndkeywordstyle=\color{darkgray}\bfseries,
  identifierstyle=\color{black},
  sensitive=false,
  comment=[l]{//},
  morecomment=[s]{/*}{*/},
  commentstyle=\color{purple}\ttfamily,
  stringstyle=\color{red}\ttfamily,
  morestring=[b]',
  morestring=[b]"
}
"#;

// https://tex.stackexchange.com/a/13761
pub const AQUOTE: &str = r#"
\def\signed #1{{\leavevmode\unskip\nobreak\hfil\penalty50\hskip2em
  \hbox{}\nobreak\hfil(#1)%
  \parfillskip=0pt \finalhyphendemerits=0 \endgraf}}

\newsavebox\mybox
\newenvironment{aquote}[1]
  {\savebox\mybox{#1}\begin{quote}}
  {\signed{\usebox\mybox}\end{quote}}
"#;

// https://journals.plos.org/plosntds/article/file?type=supplementary&id=info:doi/10.1371/journal.pntd.0007353.s002&rev=1
pub const THICKHLINE: &str = r#"
\newlength\savedwidth
\newcommand\thickhline{\noalign{\global\savedwidth\arrayrulewidth\global\arrayrulewidth 1pt}%
\hline
\noalign{\global\arrayrulewidth\savedwidth}}
"#;

// https://tex.stackexchange.com/a/160022
pub const FIX_INCLUDEGRAPHICS: &str = r#"
% Redefine \includegraphics so that, unless explicit options are
% given, the image width will not exceed the width or the height of the page.
% Images get their normal width if they fit onto the page, but
% are scaled down if they would overflow the margins.
\makeatletter
\def\ScaleWidthIfNeeded{%
 \ifdim\Gin@nat@width>\linewidth
    \linewidth
  \else
    \Gin@nat@width
  \fi
}
\def\ScaleHeightIfNeeded{%
  \ifdim\Gin@nat@height>0.9\textheight
    0.9\textheight
  \else
    \Gin@nat@width
  \fi
}
\makeatother

\setkeys{Gin}{width=\ScaleWidthIfNeeded,height=\ScaleHeightIfNeeded,keepaspectratio}
"#;

// https://tex.stackexchange.com/a/75104
pub const IMAGE_WITH_TEXT: &str = r#"
\newsavebox\imagebox
\newcommand*{\imagewithtext}[3][]{%
  \sbox\imagebox{\includegraphics[{#1}]{#2}}%
  \usebox\imagebox
  \llap{%
    \resizebox{\wd\imagebox}{\height}{%
      \texttransparent{0}{#3}%
    }%
  }%
}
"#;

// https://tex.stackexchange.com/q/183699
pub const SCALE_TIKZ_PICTURE_TO_WIDTH: &str = r#"
\makeatletter
\newsavebox{\measure@tikzpicture}
\NewEnviron{scaletikzpicturetowidth}[1]{%
  \def\tikz@width{#1}%
  \def\tikzscale{1}\begin{lrbox}{\measure@tikzpicture}%
  \BODY
  \end{lrbox}%
  \pgfmathparse{#1/\wd\measure@tikzpicture}%
  \edef\tikzscale{\pgfmathresult}%
  \BODY
}
\makeatother
"#;

// slightly modified from
// https://github.com/jpbernius/tum-thesis-latex/blob/740e69c6a9671c7c0e3d74c0a70604a0ceddde56/pages/cover.tex
pub const THESIS_COVER: &str = r#"
\begin{titlepage}
  % HACK for two-sided documents: ignore binding correction for cover page.
  % Adapted from Markus Kohm's KOMA-Script titlepage=firstiscover handling.
  % See http://mirrors.ctan.org/macros/latex/contrib/koma-script/scrkernel-title.dtx,
  % \maketitle macro.
  \oddsidemargin=\evensidemargin\relax
  \textwidth=\dimexpr\paperwidth-2\evensidemargin-2in\relax
  \hsize=\textwidth\relax

  \centering

  \ifempty{\getLogoUniversity}
    \vspace*{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoUniversity}
  \fi

  \vspace{5mm}
  {\huge\MakeUppercase{\getFaculty{}}}

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity{}}}

  \vspace{20mm}
  {\Large \getThesisType{}}

  \vspace{15mm}
  {\huge\bfseries \getTitle{}}

  \vspace{15mm}
  {\LARGE \getAuthor{}}

  \ifempty{\getLogoFaculty}
  \else
    \vspace{20mm}
    \includegraphics[height=20mm]{\getLogoFaculty}
  \fi
\end{titlepage}
"#;

// modified from
// https://github.com/waltsims/TUM_Thesis_Template_CSE/blob/2a7a2f14f7b3de8873e50d2762206a78bd872470/components/cover.tex
// TODO: l18n
pub const THESIS_TITLE: &str = r#"
\begin{titlepage}
  \centering

  \ifempty{\getLogoUniversity}
    \vspace*{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoUniversity}
  \fi

  \vspace{5mm}
  {\huge\MakeUppercase{\getFaculty{}}}

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity{}}}

  \vspace{20mm}
  {\Large \getThesisType{}}

  \vspace{15mm}
  {\huge\bfseries \getTitle{}}

  \vspace{5mm}
  {\huge\bfseries \getSubtitle{}}

  \vspace{15mm}
  \begin{tabular}{l l}
    Author: & \getAuthor{} \\
    Supervisor: & \getSupervisor{} \\
    Advisor: & \getAdvisor{} \\
    Submission Date: & \getDate{} \\
  \end{tabular}

  \ifempty{\getLogoFaculty}
    \vspace{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoFaculty}
  \fi
\end{titlepage}
"#;

// modified from
// https://github.com/jpbernius/tum-thesis-latex/blob/740e69c6a9671c7c0e3d74c0a70604a0ceddde56/pages/disclaimer.tex
pub const THESIS_DISCLAIMER: &str = r#"
\thispagestyle{empty}
\vspace*{0.75\textheight}
\noindent
\getDisclaimer

\vspace{15mm}
\noindent
\getLocation{}, \getDate{} \hspace{50mm} \getAuthor{}
"#;

// slightly modified from THESIS_COVER from
// https://github.com/jpbernius/tum-thesis-latex/blob/740e69c6a9671c7c0e3d74c0a70604a0ceddde56/pages/cover.tex
pub const REPORT_COVER: &str = r#"
\begin{titlepage}
  % HACK for two-sided documents: ignore binding correction for cover page.
  % Adapted from Markus Kohm's KOMA-Script titlepage=firstiscover handling.
  % See http://mirrors.ctan.org/macros/latex/contrib/koma-script/scrkernel-title.dtx,
  % \maketitle macro.
  \oddsidemargin=\evensidemargin\relax
  \textwidth=\dimexpr\paperwidth-2\evensidemargin-2in\relax
  \hsize=\textwidth\relax

  \centering

  \ifempty{\getLogoUniversity}
    \vspace*{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoUniversity}
  \fi

  \vspace{5mm}
  \ifempty{\getFaculty}
    \vspace*{1em}
  \else
    {\huge\MakeUppercase{\getFaculty}}
  \fi

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity}}

  \vspace{15mm}
  {\huge\bfseries \getTitle{}}

  \vspace{5mm}
  \ifempty{\getSubtitle}
    {\huge\vspace{1em}}
  \else
    {\huge\bfseries \getSubtitle{}}
  \fi

  \vspace{15mm}
  {\LARGE \getAuthor{}}

  \ifempty{\getLogoFaculty}
    \vspace{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoFaculty}
  \fi
\end{titlepage}
"#;

// https://tex.stackexchange.com/a/97188
// https://tex.stackexchange.com/a/343329
pub const TABULARX: &str = r#"
\newcolumntype{L}{>{\raggedright\let\newline\\\arraybackslash\hspace{0pt}}X}
\newcolumntype{C}{>{\centering\let\newline\\\arraybackslash\hspace{0pt}}X}
\newcolumntype{R}{>{\raggedleft\let\newline\\\arraybackslash\hspace{0pt}}X}
\renewcommand\tabularxcolumn[1]{m{#1}}
"#;
