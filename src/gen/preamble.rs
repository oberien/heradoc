#![allow(non_upper_case_globals)]

// https://en.wikibooks.org/wiki/LaTeX/Source_Code_Listings
pub const lstset: &'static str = r#"
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
pub const lstdefineasm: &'static str = r#"
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

pub const lstdefinerust: &'static str = r#"
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
pub const aquote: &'static str = r#"
\def\signed #1{{\leavevmode\unskip\nobreak\hfil\penalty50\hskip2em
  \hbox{}\nobreak\hfil(#1)%
  \parfillskip=0pt \finalhyphendemerits=0 \endgraf}}

\newsavebox\mybox
\newenvironment{aquote}[1]
  {\savebox\mybox{#1}\begin{quote}}
  {\signed{\usebox\mybox}\end{quote}}
"#;
