use std::io::{Write, Result};

use isolang::Language;

use crate::config::Config;

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

    // geometry
    write!(out, "\\usepackage[")?;
    cfg.geometry.write_latex_options(&mut *out)?;
    writeln!(out, "]{{geometry}}")?;

    writeln!(out)?;

    // TODO: biblatex options (natbib?)
    if let Some(bibliography) = &cfg.bibliography {
        writeln!(out, "\\usepackage[backend=biber,citestyle={},bibstyle={}]{{biblatex}}", cfg.citestyle, cfg.bibstyle)?;
        writeln!(out, "\\addbibresource{{{}}}", bibliography.display())?;
    }

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
    // TODO: graphicspath
    writeln!(out, "\\usepackage{{graphicx}}")?;
    writeln!(out, "\\usepackage[final]{{microtype}}")?;
    writeln!(out, "\\usepackage[pdfusetitle]{{hyperref}}")?;
    writeln!(out, "\\usepackage{{caption}}")?;
    writeln!(out, "\\usepackage{{caption}}")?;
    // TODO: cleveref options
    writeln!(out, "\\usepackage{{cleveref}}")?;
    writeln!(out, "\\usepackage{{refcount}}")?;
    writeln!(out, "\\usepackage[titletoc,toc,title]{{appendix}}")?;
    writeln!(out, "\\usepackage{{array}}")?;
    writeln!(out)?;
    Ok(())
}

pub fn write_fixes(cfg: &Config, out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\setlength{{\\parindent}}{{0pt}}")?;
    writeln!(out, "\\setlength{{\\parskip}}{{1\\baselineskip plus 2pt minus 2pt}}")?;
    writeln!(out, "{}", LSTSET)?;
    writeln!(out, "{}", LST_DEFINE_ASM)?;
    writeln!(out, "{}", LST_DEFINE_RUST)?;
    writeln!(out, "{}", THICKHLINE)?;
    writeln!(out, "{}", AQUOTE)?;
    writeln!(out, "{}", FIX_INCLUDEGRAPHICS)?;
    writeln!(out, "{}", SCALE_TIKZ_PICTURE_TO_WIDTH)?;
    // TODO: figures inline? https://tex.stackexchange.com/a/11342 last codeblock
    // with package float and `[H]`

    for include in &cfg.header_includes {
        writeln!(out, "{}", include)?;
    }
    Ok(())
}

// https://en.wikibooks.org/wiki/LaTeX/Source_Code_Listings
pub const LSTSET: &'static str = r#"
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
  escapechar=ß,
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
pub const LST_DEFINE_ASM: &'static str = r#"
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

pub const LST_DEFINE_RUST: &'static str = r#"
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

// https://tex.stackexchange.com/a/13761
pub const AQUOTE: &'static str = r#"
\def\signed #1{{\leavevmode\unskip\nobreak\hfil\penalty50\hskip2em
  \hbox{}\nobreak\hfil(#1)%
  \parfillskip=0pt \finalhyphendemerits=0 \endgraf}}

\newsavebox\mybox
\newenvironment{aquote}[1]
  {\savebox\mybox{#1}\begin{quote}}
  {\signed{\usebox\mybox}\end{quote}}
"#;

// https://tex.stackexchange.com/a/41761
pub const THICKHLINE: &'static str = r#"
\makeatletter
\newcommand{\thickhline}{%
    \noalign {\ifnum 0=`}\fi \hrule height 1pt
    \futurelet \reserved@a \@xhline
}
\newcolumntype{"}{@{\hskip\tabcolsep\vrule width 1pt\hskip\tabcolsep}}
\makeatother
"#;

// https://tex.stackexchange.com/a/160022
pub const FIX_INCLUDEGRAPHICS: &'static str = r#"
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

// https://tex.stackexchange.com/q/183699
pub const SCALE_TIKZ_PICTURE_TO_WIDTH: &'static str = r#"
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
pub const THESIS_COVER: &'static str = r#"
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
  {\huge\MakeUppercase{\getFaculty{}}}\\

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity{}}}\\

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
pub const THESIS_TITLE: &'static str = r#"
\begin{titlepage}
  \centering

  \ifempty{\getLogoUniversity}
    \vspace*{20mm}
  \else
    \includegraphics[height=20mm]{\getLogoUniversity}
  \fi

  \vspace{5mm}
  {\huge\MakeUppercase{\getFaculty{}}}\\

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity{}}}\\

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
  \else
    \vspace{20mm}
    \includegraphics[height=20mm]{\getLogoFaculty}
  \fi
\end{titlepage}
"#;

// modified from
// https://github.com/jpbernius/tum-thesis-latex/blob/740e69c6a9671c7c0e3d74c0a70604a0ceddde56/pages/disclaimer.tex
pub const THESIS_DISCLAIMER: &'static str = r#"
\thispagestyle{empty}
\vspace*{0.75\textheight}
\noindent
\getDisclaimer

\vspace{15mm}
\noindent
\getLocation{}, \getDate{} \hspace{50mm} \getAuthor{}
"#;
