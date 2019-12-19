use std::io::{Result, Write};

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
        writeln!(
            out,
            "\\usepackage[backend=biber,citestyle={},bibstyle={}]{{biblatex}}",
            cfg.citestyle, cfg.bibstyle
        )?;
        writeln!(out, "\\addbibresource{{{}}}", bibliography.display())?;
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
    writeln!(out, "\\usepackage{{amsthm}}")?;
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
    writeln!(out, "\\usepackage{{cleveref}}")?;
    writeln!(out, "\\usepackage{{refcount}}")?;
    writeln!(out, "\\usepackage[titletoc,toc,title]{{appendix}}")?;
    writeln!(out, "\\usepackage{{array}}")?;
    writeln!(out, "\\usepackage{{pdfcomment}}")?;
    writeln!(out, "\\usepackage{{tabularx}}")?;
    writeln!(out)?;
    Ok(())
}

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
    writeln!(out, "{}", AMSTHM_DEFAULTS)?;
    // TODO: figures inline? https://tex.stackexchange.com/a/11342 last codeblock
    // with package float and `[H]`

    for include in &cfg.header_includes {
        writeln!(out, "{}", include)?;
    }
    Ok(())
}

pub fn write_university_commands(cfg: &Config, out: &mut impl Write) -> Result<()> {
    fn get(o: &Option<String>) -> &str {
        o.as_ref().map_or("", |s| s.as_str())
    }
    writeln!(out, "\\newcommand*{{\\getTitle}}{{{}}}", get(&cfg.title))?;
    writeln!(out, "\\newcommand*{{\\getSubtitle}}{{{}}}", get(&cfg.subtitle))?;
    writeln!(out, "\\newcommand*{{\\getAuthor}}{{{}}}", get(&cfg.author))?;
    writeln!(out, "\\newcommand*{{\\getDate}}{{{}}}", get(&cfg.date))?;
    writeln!(out, "\\newcommand*{{\\getSupervisor}}{{{}}}", get(&cfg.supervisor))?;
    writeln!(out, "\\newcommand*{{\\getAdvisor}}{{{}}}", get(&cfg.advisor))?;
    if let Some(logo_university) = cfg.logo_university.as_ref() {
        writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{{}}}", logo_university.display())?;
    } else {
        writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{}}")?;
    }
    if let Some(logo_faculty) = cfg.logo_faculty.as_ref() {
        writeln!(out, "\\newcommand*{{\\getLogoFaculty}}{{{}}}", logo_faculty.display())?;
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
  keywords={typeof, new, true, false, catch, function, return, null, catch, switch, var, if, in, while, do, else, case, break},
  keywordstyle=\color{blue}\bfseries,
  ndkeywords={class, export, boolean, throw, implements, import, this},
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

// https://tex.stackexchange.com/a/41761
pub const THICKHLINE: &str = r#"
\makeatletter
\newcommand{\thickhline}{%
    \noalign {\ifnum 0=`}\fi \hrule height 1pt
    \futurelet \reserved@a \@xhline
}
\newcolumntype{"}{@{\hskip\tabcolsep\vrule width 1pt\hskip\tabcolsep}}
\makeatother
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
pub const THESIS_TITLE: &str = r#"
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
    {\huge\MakeUppercase{\getFaculty}}\\
  \fi

  \vspace{5mm}
  {\large\MakeUppercase{\getUniversity}}\\

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

// TODO: i18n? How to interact with this smartly?
pub const AMSTHM_DEFAULTS: &str = r#"
\newcounter{common-thm-ctr}
\theoremstyle{plain}
\newtheorem{amsthm-theorem}[common-thm-ctr]{Theorem}
\newtheorem{amsthm-lemma}[common-thm-ctr]{Lemma}
\newtheorem{amsthm-corollary}[common-thm-ctr]{Corollary}
\theoremstyle{amsthm-definition}
\newtheorem{amsthm-definition}[common-thm-ctr]{Definition}
"#;
