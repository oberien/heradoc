use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write, BufWriter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use boolinator::Boolinator;
use isolang::Language;
use serde::{de, Deserialize, Deserializer};
use structopt::StructOpt;
use strum_macros::{Display, EnumString};
use tempdir::TempDir;
use url::Url;
use void::Void;

mod geometry;

use self::geometry::Geometry;
use crate::resolve::remote::Remote;
use crate::util;

// TODO: VecOrSingle to allow `foo = "bar"` instead of `foo = ["bar"]` for single values

#[derive(StructOpt, Debug)]
#[structopt(name = "heradoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct CliArgs {
    /// Output file. Use `-` for stdout.
    #[structopt(short = "o", long = "out", long = "output")]
    pub output: Option<FileOrStdio>,
    /// Output directory for itermediate files. Defaults to a tempdir.
    #[structopt(long = "outdir", parse(from_os_str))]
    pub out_dir: Option<PathBuf>,
    /// Input markdown file. Use `-` for stdin.
    #[structopt()]
    pub input: FileOrStdio,
    /// Config file with additional configuration. Defaults to `Config.toml` if it exists.
    #[structopt(long = "config", long = "cfg", parse(from_os_str))]
    pub configfile: Option<PathBuf>,
    #[structopt(flatten)]
    pub fileconfig: FileConfig,
}

#[derive(Debug, Default, Deserialize, StructOpt)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct FileConfig {
    /// Output type (tex / pdf). If left blank, it's derived from the output file ending.
    /// Defaults to tex for stdout.
    #[structopt(short = "t", long = "to", long = "out-type", long = "output-type")]
    pub output_type: Option<OutType>,

    /// Type of the document.
    #[structopt(long)]
    pub document_type: Option<DocumentType>,

    // TODO: multiple files (VecOrSingle)
    /// Bibliography file in biblatex format. Defaults to references.bib (if it exists).
    #[structopt(long)]
    pub bibliography: Option<String>,
    /// Citation style. Used for both `citestyle` and `bibstyle`.
    #[structopt(long)]
    pub citationstyle: Option<MaybeUnknown<CitationStyle>>,
    /// Style used for citation labels. Takes precedence over `citationstyle`.
    #[structopt(long)]
    pub citestyle: Option<MaybeUnknown<CitationStyle>>,
    /// Style used for generating the bibliography index. Takes precedence over `citationstyle`.
    #[structopt(long)]
    pub bibstyle: Option<MaybeUnknown<CitationStyle>>,
    /// If figures should be used by default for images and similar
    #[structopt(long)]
    pub figures: Option<bool>,

    /// File template to use. It must contain `HERADOCBODY` on its own line without indentation,
    /// which will be replaced with the rendered body.
    /// If this parameter is used, other header-configuration options will be discarded.
    #[structopt(long)]
    pub template: Option<String>,

    /// Language of document (used for l18n).
    /// Can be an ISO 639-1 or ISO 639-3 identifier (e.g. "en"), or a locale (e.g. "de_DE").
    /// Defaults to english.
    #[structopt(long = "lang", long = "language")]
    pub lang: Option<String>,

    /// Fontsize of the document.
    #[structopt(long)]
    pub fontsize: Option<String>,
    /// When rendering a book sets whether the book should be one-sided or two-sided
    #[structopt(long)]
    pub oneside: Option<bool>,
    /// Other options passed to `\documentclass`.
    #[structopt(long)]
    #[serde(default)]
    pub classoptions: Vec<String>,

    // titlepage
    /// For article if true, the titlepage will be its own page. Otherwise text will start on the
    /// first page.
    #[structopt(long)]
    pub titlepage: Option<bool>,
    /// Title of document, used for titlepage
    #[structopt(long)]
    pub title: Option<String>,
    /// Subitle of document, used for titlepage
    #[structopt(long)]
    pub subtitle: Option<String>,
    /// Titlehead of the titlepage
    #[structopt(long)]
    pub titlehead: Option<String>,
    /// Author(s) of document, used for titlepage
    #[structopt(long)]
    pub author: Option<String>,
    /// Email(s) of authors, used for titlepage
    #[structopt(long)]
    pub email: Option<String>,
    /// Date of document, used for titlepage
    #[structopt(long)]
    pub date: Option<String>,
    /// Publisher of document, used for titlepage of article
    #[structopt(long)]
    pub publisher: Option<String>,
    /// Advisor of document, used for titlepage
    #[structopt(long)]
    pub advisor: Option<String>,
    /// Supervisor of document, used for titlepage
    #[structopt(long)]
    pub supervisor: Option<String>,
    // only for thesis
    /// University Logo, displayed on top of titlepage of theses
    #[structopt(long)]
    pub logo_university: Option<String>,
    /// Faculty logo, displayed on bottom of titlepage of theses
    #[structopt(long)]
    pub logo_faculty: Option<String>,
    /// University name
    #[structopt(long)]
    pub university: Option<String>,
    /// Faculty name
    #[structopt(long)]
    pub faculty: Option<String>,
    /// Thesis type (e.g. "Master's Thesis in Informatics")
    #[structopt(long)]
    pub thesis_type: Option<String>,
    /// Submission Location of the thesis
    #[structopt(long)]
    pub location: Option<String>,
    /// Disclaimer for theses
    #[structopt(long)]
    pub disclaimer: Option<String>,
    /// Path to markdown file containing the abstract.
    #[structopt(long = "abstract")]
    #[serde(rename = "abstract")]
    pub abstract1: Option<String>,
    /// Path to a second file containing the abstract in a different language.
    #[structopt(long)]
    pub abstract2: Option<String>,

    // fancyhdr
    /// Left-aligned header
    #[structopt(long)]
    pub lhead: Option<String>,
    /// Left-aligned header on even pages (used if oneside=false; if not specified, lhead is used for all pages)
    #[structopt(long)]
    pub lhead_even: Option<String>,
    /// Center-aligned header
    #[structopt(long)]
    pub chead: Option<String>,
    /// Center-aligned header on even pages (used if oneside=false; if not specified, chead is used for all pages)
    #[structopt(long)]
    pub chead_even: Option<String>,
    /// Right-aligned header
    #[structopt(long)]
    pub rhead: Option<String>,
    /// Right-aligned header on even pages (used if oneside=false; if not specified, rhead is used for all pages)
    #[structopt(long)]
    pub rhead_even: Option<String>,
    /// Left-aligned footer
    #[structopt(long)]
    pub lfoot: Option<String>,
    /// Left-aligned footer on even pages (used if oneside=false; if not specified, lfoot is used for all pages)
    #[structopt(long)]
    pub lfoot_even: Option<String>,
    /// Center-aligned footer
    #[structopt(long)]
    pub cfoot: Option<String>,
    /// Center-aligned footer on even pages (used if oneside=false; if not specified, cfoot is used for all pages)
    #[structopt(long)]
    pub cfoot_even: Option<String>,
    /// Right-aligned footer
    #[structopt(long)]
    pub rfoot: Option<String>,
    /// Right-aligned footer on even pages (used if oneside=false; if not specified, rfoot is used for all pages)
    #[structopt(long)]
    pub rfoot_even: Option<String>,
    /// If the header and footer should be visible on the titlepage
    #[structopt(long)]
    pub header_footer_on_titlepage: Option<bool>,

    // only for beamer
    /// If true, inserts titleframes for each section before starting with that section's content frames
    #[structopt(long)]
    pub sectionframes: Option<bool>,
    /// Theme to use for beamer presentations. Defaults to Madrid.
    #[structopt(long)]
    pub beamertheme: Option<String>,

    /// Custom header includes
    #[structopt(long)]
    #[serde(default)]
    pub header_includes: Vec<String>,

    // geometry
    #[structopt(flatten)]
    #[serde(default)]
    pub geometry: Geometry,
}

#[derive(Debug)]
// TODO: make strongly typed
pub struct Config {
    // IO
    pub output: FileOrStdio,
    pub out_dir: PathBuf,
    /// Space for auxiliary files that *must not* be accessible directly.
    ///
    /// In particular, it should not be possible to reference a markdown file placed in this
    /// directory.  This prevents content injection from untrusted sources and is currently the
    /// result of choosing this path randomly.  TODO: Make this restriction explicit.
    pub temp_dir: PathBuf,
    pub input: FileOrStdio,
    pub document_folder: PathBuf,
    pub project_root: PathBuf,
    pub output_type: OutType,

    pub document_type: DocumentType,

    pub bibliography: Option<PathBuf>,
    pub citestyle: MaybeUnknown<CitationStyle>,
    pub bibstyle: MaybeUnknown<CitationStyle>,
    pub figures: bool,

    pub template: Option<PathBuf>,

    pub lang: Language,

    // document
    pub fontsize: String,
    pub oneside: bool,
    pub classoptions: HashSet<String>,

    // titlepage
    pub titlepage: bool,
    // metadata
    // TODO: make metadata content dependent on document_type
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub titlehead: Option<String>,
    pub author: Option<String>,
    pub email: Option<String>,
    pub date: Option<String>,
    pub publisher: Option<String>,
    pub advisor: Option<String>,
    pub supervisor: Option<String>,
    // only for thesis
    pub logo_university: Option<PathBuf>,
    pub logo_faculty: Option<PathBuf>,
    pub university: Option<String>,
    pub faculty: Option<String>,
    pub thesis_type: Option<String>,
    pub location: Option<String>,
    pub disclaimer: Option<String>,
    pub abstract1: Option<PathBuf>,
    pub abstract2: Option<PathBuf>,
    // fancyhdr
    pub lhead: Option<String>,
    pub lhead_even: Option<String>,
    pub chead: Option<String>,
    pub chead_even: Option<String>,
    pub rhead: Option<String>,
    pub rhead_even: Option<String>,
    pub lfoot: Option<String>,
    pub lfoot_even: Option<String>,
    pub cfoot: Option<String>,
    pub cfoot_even: Option<String>,
    pub rfoot: Option<String>,
    pub rfoot_even: Option<String>,
    pub header_footer_on_titlepage: bool,
    // only for beamer
    pub sectionframes: bool,
    pub beamertheme: String,

    pub header_includes: Vec<String>,

    // geometry
    pub geometry: Geometry,
}

impl Config {
    /// tempdir must live as long as Config
    pub fn new(args: CliArgs, infile: FileConfig, file: FileConfig, cfgfile_folder: Option<PathBuf>, tempdir: &TempDir) -> Config {
        let tempdir_path = tempdir.path().to_owned();
        // verify input file
        match &args.input {
            FileOrStdio::StdIo => (),
            FileOrStdio::File(path) if path.is_file() => (),
            FileOrStdio::File(path) => panic!("Invalid File {:?}", path),
        }
        // cli > infile > configfile
        let output_type =
            match args.fileconfig.output_type.or(infile.output_type).or(file.output_type) {
                Some(typ) => typ,
                None => match &args.output {
                    Some(FileOrStdio::StdIo) => OutType::Latex,
                    Some(FileOrStdio::File(path)) => path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .and_then(|ext| {
                            (ext.eq_ignore_ascii_case("tex") || ext.eq_ignore_ascii_case("latex"))
                                .as_some(OutType::Latex)
                        })
                        .unwrap_or(OutType::Pdf),
                    None => OutType::Pdf,
                },
            };
        let output = match args.output {
            Some(fos) => fos,
            None => {
                let mut filename = match &args.input {
                    FileOrStdio::File(path) => PathBuf::from(path.file_stem().unwrap()),
                    FileOrStdio::StdIo => PathBuf::from("out"),
                };
                match output_type {
                    OutType::Latex => assert!(filename.set_extension("tex")),
                    OutType::Pdf => assert!(filename.set_extension("pdf")),
                    OutType::Mp4 => assert!(filename.set_extension("mp4")),
                }
                FileOrStdio::File(filename)
            },
        };

        let document_folder = args.input.folder_canonicalized()
            .unwrap_or_else(|| {
                env::current_dir().expect("Can't use stdin without a current working directory")
            });
        let project_root = cfgfile_folder.unwrap_or_else(|| document_folder.clone());

        let bibliography = args
            .fileconfig
            .bibliography
            .or(infile.bibliography)
            .or(file.bibliography)
            .or_else(|| {
                if Path::new("references.bib").is_file() {
                    Some("references.bib".to_string())
                } else {
                    None
                }
            });
        let bibliography = resolve_file(&document_folder, &project_root, &tempdir_path, bibliography, "bibliography");
        let template =
            args.fileconfig.template.or(infile.template).or(file.template);
        let template = resolve_file(&document_folder, &project_root, &tempdir_path, template, "template");

        let lang = args.fileconfig.lang.or(infile.lang).or(file.lang);
        let lang = match lang {
            None => Language::Eng,
            Some(lang) => Language::from_639_1(&lang)
                .or_else(|| Language::from_639_3(&lang))
                .or_else(|| Language::from_locale(&lang))
                // TODO: improve error message (with origin and value)
                .expect("Unknown language parameter"),
        };

        let mut classoptions = HashSet::new();
        classoptions.extend(args.fileconfig.classoptions);
        classoptions.extend(infile.classoptions);
        classoptions.extend(file.classoptions);

        let mut header_includes = args.fileconfig.header_includes;
        header_includes.extend(infile.header_includes);
        header_includes.extend(file.header_includes);

        let citationstyle =
            args.fileconfig.citationstyle.or(infile.citationstyle).or(file.citationstyle);

        let logo_university = args
            .fileconfig
            .logo_university
            .or(infile.logo_university)
            .or(file.logo_university);
        let logo_university =
            resolve_file(&document_folder, &project_root, &tempdir_path, logo_university, "logo_university");
        let logo_faculty = args
            .fileconfig
            .logo_faculty
            .or(infile.logo_faculty)
            .or(file.logo_faculty);
        let logo_faculty = resolve_file(&document_folder, &project_root, &tempdir_path, logo_faculty, "logo_faculty");
        let abstract1 =
            args.fileconfig.abstract1.or(infile.abstract1).or(file.abstract1);
        let abstract1 = resolve_file(&document_folder, &project_root, &tempdir_path, abstract1, "abstract");
        let abstract2 =
            args.fileconfig.abstract2.or(infile.abstract2).or(file.abstract2);
        let abstract2 = resolve_file(&document_folder, &project_root, &tempdir_path, abstract2, "abstract2");

        let document_type = args
            .fileconfig
            .document_type
            .or(infile.document_type)
            .or(file.document_type)
            .unwrap_or(DocumentType::Article);

        Config {
            output,
            out_dir: args.out_dir.unwrap_or_else(|| tempdir.path().to_owned()),
            temp_dir: tempdir_path,
            input: args.input,
            document_folder,
            project_root,
            output_type,
            document_type,
            bibliography,
            template,
            lang,
            citestyle: args
                .fileconfig
                .citestyle
                .or(infile.citestyle)
                .or(file.citestyle)
                .or_else(|| citationstyle.as_ref().cloned())
                .unwrap_or(MaybeUnknown::Known(CitationStyle::NumericComp)),
            bibstyle: args
                .fileconfig
                .bibstyle
                .or(infile.bibstyle)
                .or(file.bibstyle)
                .or(citationstyle)
                .unwrap_or(MaybeUnknown::Known(CitationStyle::Ieee)),
            figures: args.fileconfig.figures.or(infile.figures).or(file.figures).unwrap_or_else(
                || match document_type {
                    DocumentType::Article | DocumentType::Beamer => false,
                    DocumentType::Thesis | DocumentType::Report => true,
                },
            ),
            fontsize: args
                .fileconfig
                .fontsize
                .or(infile.fontsize)
                .or(file.fontsize)
                .unwrap_or_else(|| "11pt".to_string()),
            oneside: args.fileconfig.oneside.or(infile.oneside).or(file.oneside).unwrap_or(false),
            titlepage: args
                .fileconfig
                .titlepage
                .or(infile.titlepage)
                .or(file.titlepage)
                .unwrap_or(true),
            title: args.fileconfig.title.or(infile.title).or(file.title),
            subtitle: args.fileconfig.subtitle.or(infile.subtitle).or(file.subtitle),
            titlehead: args.fileconfig.titlehead.or(infile.titlehead).or(file.titlehead),
            author: args.fileconfig.author.or(infile.author).or(file.author),
            email: args.fileconfig.email.or(infile.email).or(file.email),
            date: args.fileconfig.date.or(infile.date).or(file.date),
            publisher: args.fileconfig.publisher.or(infile.publisher).or(file.publisher),
            advisor: args.fileconfig.advisor.or(infile.advisor).or(file.advisor),
            supervisor: args.fileconfig.supervisor.or(infile.supervisor).or(file.supervisor),
            logo_university,
            logo_faculty,
            university: args.fileconfig.university.or(infile.university).or(file.university),
            faculty: args.fileconfig.faculty.or(infile.faculty).or(file.faculty),
            thesis_type: args.fileconfig.thesis_type.or(infile.thesis_type).or(file.thesis_type),
            location: args.fileconfig.location.or(infile.location).or(file.location),
            disclaimer: args.fileconfig.disclaimer.or(infile.disclaimer).or(file.disclaimer),
            abstract1,
            abstract2,
            lhead: args.fileconfig.lhead.or(infile.lhead).or(file.lhead),
            lhead_even: args.fileconfig.lhead_even.or(infile.lhead_even).or(file.lhead_even),
            chead: args.fileconfig.chead.or(infile.chead).or(file.chead),
            chead_even: args.fileconfig.chead_even.or(infile.chead_even).or(file.chead_even),
            rhead: args.fileconfig.rhead.or(infile.rhead).or(file.rhead),
            rhead_even: args.fileconfig.rhead_even.or(infile.rhead_even).or(file.rhead_even),
            lfoot: args.fileconfig.lfoot.or(infile.lfoot).or(file.lfoot),
            lfoot_even: args.fileconfig.lfoot_even.or(infile.lfoot_even).or(file.lfoot_even),
            cfoot: args.fileconfig.cfoot.or(infile.cfoot).or(file.cfoot),
            cfoot_even: args.fileconfig.cfoot_even.or(infile.cfoot_even).or(file.cfoot_even),
            rfoot: args.fileconfig.rfoot.or(infile.rfoot).or(file.rfoot),
            rfoot_even: args.fileconfig.rfoot_even.or(infile.rfoot_even).or(file.rfoot_even),
            header_footer_on_titlepage: args.fileconfig.header_footer_on_titlepage.or(infile.header_footer_on_titlepage).or(file.header_footer_on_titlepage).unwrap_or(false),
            sectionframes: args.fileconfig.sectionframes.or(infile.sectionframes).or(file.sectionframes).unwrap_or(true),
            beamertheme: args.fileconfig.beamertheme.or(infile.beamertheme).or(file.beamertheme).unwrap_or_else(|| "Madrid".to_string()),
            classoptions,
            header_includes,
            geometry: args.fileconfig.geometry.merge(infile.geometry).merge(file.geometry),
        }
    }
}

/// Tries to resolve given input.
///
/// 1. if it's relative, it's resolved relative to the input file
/// 2. if it's absolute, it's resolved relative to the project root
/// 3. if it's a URL, the content will be downloaded and a path to the downloaded file returned.
fn resolve_file<P: AsRef<str>>(
    document_folder: &Path, project_root: &Path, temp_dir: &Path, to_resolve: Option<P>, cfgoption_name: &str,
) -> Option<PathBuf> {
    // TODO: error handling
    let to_resolve = to_resolve?;
    let to_resolve = to_resolve.as_ref();
    let path = Path::new(to_resolve);

    // relative to input file
    if path.is_relative() {
        let file = document_folder.join(&path);
        if file.exists() && file.is_file() {
            return Some(file);
        }
    }
    // relative to project root
    if path.is_absolute() {
        let file = project_root.join(util::strip_root(path));
        if file.exists() && file.is_file() {
            return Some(file);
        }
    }

    // try to download
    let remote = Remote::new(temp_dir.to_owned()).unwrap();
    match remote.http(&Url::parse(to_resolve).unwrap()) {
        Err(_) => panic!("{} file doesn't exist or isn't a url: {:?}", cfgoption_name, to_resolve),
        Ok(downloaded) => Some(downloaded.path().to_owned()),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum MaybeUnknown<T> {
    Known(T),
    Unknown(String),
}

impl<T: Default> Default for MaybeUnknown<T> {
    fn default() -> Self {
        MaybeUnknown::Known(T::default())
    }
}

impl<T: FromStr> FromStr for MaybeUnknown<T> {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Void> {
        Ok(T::from_str(s)
            .map(MaybeUnknown::Known)
            .unwrap_or_else(|_| MaybeUnknown::Unknown(s.to_string())))
    }
}

impl<T: fmt::Display> fmt::Display for MaybeUnknown<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeUnknown::Known(t) => t.fmt(f),
            MaybeUnknown::Unknown(s) => s.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum FileOrStdio {
    StdIo,
    File(PathBuf),
}

impl FileOrStdio {
    pub fn to_read(&self) -> Box<dyn Read> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdin())).lock()),
            FileOrStdio::File(path) => Box::new(File::open(path).expect("can't open input source")),
        }
    }

    pub fn to_write(&self) -> Box<dyn Write> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdout())).lock()),
            FileOrStdio::File(path) => {
                Box::new(BufWriter::new(File::create(path).expect("can't open output source")))
            },
        }
    }

    pub fn folder_canonicalized(&self) -> Option<PathBuf> {
        match self {
            FileOrStdio::StdIo => None,
            FileOrStdio::File(file) => Some(file
                .canonicalize()
                .expect("error canonicalizing input file path")
                .parent()
                .unwrap()
                .to_owned())
        }
    }
}

impl FromStr for FileOrStdio {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" | "-" => Ok(FileOrStdio::StdIo),
            s => Ok(FileOrStdio::File(PathBuf::from(s))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutType {
    Latex,
    Pdf,
    Mp4,
}

impl<'de> Deserialize<'de> for OutType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl FromStr for OutType {
    type Err = String;

    fn from_str(s: &str) -> Result<OutType, Self::Err> {
        if s.eq_ignore_ascii_case("tex") || s.eq_ignore_ascii_case("latex") {
            Ok(OutType::Latex)
        } else if s.eq_ignore_ascii_case("pdf") {
            Ok(OutType::Pdf)
        } else if s.eq_ignore_ascii_case("mp4") || s.eq_ignore_ascii_case("ffmpeg") {
            Ok(OutType::Mp4)
        } else {
            Err(format!("unknown output type {:?}", s))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumString)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
#[strum(serialize_all = "kebab_case")]
pub enum DocumentType {
    Article,
    Report,
    Thesis,
    Beamer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display, EnumString)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[strum(serialize_all = "kebab_case")]
pub enum CitationStyle {
    Numeric,
    NumericComp,
    NumericVerb,
    Alphabetic,
    AlphabeticVerb,
    Authoryear,
    AuthoryearComp,
    AuthoryearIbid,
    AuthoryearIcomp,
    Authortitle,
    AuthortitleComp,
    AuthortitleIbid,
    AuthortitleIcomp,
    AuthortitleTerse,
    AuthortitleTcomp,
    AuthortitleTicomp,
    Verbose,
    VerboseIbid,
    VerboseNote,
    VerboseInode,
    VerboseTrad1,
    VerboseTrad2,
    VerboseTrad3,
    Reading,
    Draft,
    Debug,
    // non-standard
    ChemAcs,
    Phys,
    Nature,
    Science,
    Ieee,
    ChicagoAuthordate,
    Mla,
    Apa,
}
