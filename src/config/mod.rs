use std::path::{PathBuf, Path};
use std::fs::File;
use std::str::FromStr;
use std::io::{self, Read, Write};
use std::fmt;
use std::collections::HashSet;
use std::env;

use void::Void;
use boolinator::Boolinator;
use structopt::StructOpt;
use serde::{Deserialize, Deserializer, de};
use tempdir::TempDir;
use isolang::Language;

mod geometry;

use self::geometry::Geometry;

// TODO: VecOrSingle to allow `foo = "bar"` instead of `foo = ["bar"]` for single values

#[derive(StructOpt, Debug)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct CliArgs {
    /// Output file. Use `-` for stdout.
    #[structopt(short = "o", long = "out", long = "output")]
    pub output: Option<FileOrStdio>,
    /// Output directory for itermediate files. Defaults to a tempdir.
    #[structopt(long="outdir", parse(from_os_str))]
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
pub struct FileConfig {
    /// Output type (tex / pdf). If left blank, it's derived from the output file ending.
    /// Defaults to tex for stdout.
    #[structopt(short = "t", long = "to", long = "out-type", long = "output-type")]
    pub output_type: Option<OutType>,

    /// Type of the document.
    #[structopt(long = "document-type", long = "doc-type")]
    pub document_type: Option<DocumentType>,

    // TODO: multiple files (VecOrSingle)
    /// Bibliography file in biblatex format. Defaults to references.bib (if it exists).
    #[structopt(long = "bibliography")]
    pub bibliography: Option<String>,
    /// Citation style. Used for both `citestyle` and `bibstyle`.
    #[structopt(long = "citationstyle")]
    pub citationstyle: Option<MaybeUnknown<CitationStyle>>,
    /// Style used for citation labels. Takes precedence over `citationstyle`.
    #[structopt(long = "citestyle")]
    pub citestyle: Option<MaybeUnknown<CitationStyle>>,
    /// Style used for generating the bibliography index. Takes precedence over `citationstyle`.
    #[structopt(long = "bibstyle")]
    pub bibstyle: Option<MaybeUnknown<CitationStyle>>,
    /// If figures should be used by default for images and similar
    #[structopt(long = "figures")]
    pub figures: Option<bool>,

    /// File template to use. It must contain `PUNDOCBODY` on its own line without indentation,
    /// which will be replaced with the rendered body.
    /// If this parameter is used, other header-configuration options will be discarded.
    #[structopt(long = "template")]
    pub template: Option<String>,

    /// Language of document (used for l18n).
    /// Can be an ISO 639-1 or ISO 639-3 identifier (e.g. "en"), or a locale (e.g. "de_DE").
    /// Defaults to english.
    #[structopt(long = "lang", long = "language")]
    pub lang: Option<String>,

    /// Fontsize of the document.
    #[structopt(long = "fontsize")]
    pub fontsize: Option<String>,
    /// When rendering a book sets whether the book should be one-sided or two-sided
    #[structopt(long = "oneside")]
    pub oneside: Option<bool>,
    /// Other options passed to `\documentclass`.
    #[structopt(long = "classoptions")]
    #[serde(default)]
    pub classoptions: Vec<String>,

    // titlepage
    /// For article if true, the titlepage will be its own page. Otherwise text will start on the first page.
    #[structopt(long = "titlepage")]
    pub titlepage: Option<bool>,
    /// Title of document, used for titlepage
    #[structopt(long = "title")]
    pub title: Option<String>,
    /// Subitle of document, used for titlepage
    #[structopt(long = "subtitle")]
    pub subtitle: Option<String>,
    /// Author(s) of document, used for titlepage
    #[structopt(long = "author")]
    pub author: Option<String>,
    /// Date of document, used for titlepage
    #[structopt(long = "date")]
    pub date: Option<String>,
    /// Publisher of document, used for titlepage of article
    #[structopt(long = "publisher")]
    pub publisher: Option<String>,
    /// Advisor of document, used for titlepage
    #[structopt(long = "advisor")]
    pub advisor: Option<String>,
    /// Supervisor of document, used for titlepage
    #[structopt(long = "supervisor")]
    pub supervisor: Option<String>,
    // only for thesis
    /// University Logo, displayed on top of titlepage of theses
    #[structopt(long = "logo-university")]
    pub logo_university: Option<String>,
    /// Faculty logo, displayed on bottom of titlepage of theses
    #[structopt(long = "logo-faculty")]
    pub logo_faculty: Option<String>,
    /// University name
    #[structopt(long = "university")]
    pub university: Option<String>,
    /// Faculty name
    #[structopt(long = "faculty")]
    pub faculty: Option<String>,
    /// Thesis type (e.g. "Master's Thesis in Informatics")
    #[structopt(long = "thesis-type")]
    pub thesis_type: Option<String>,
    /// Submission Location of the thesis
    #[structopt(long = "location")]
    pub location: Option<String>,
    /// Disclaimer for theses
    #[structopt(long = "disclaimer")]
    pub disclaimer: Option<String>,
    /// Path to markdown file containing the abstract.
    #[structopt(long = "abstract")]
    #[serde(rename = "abstract")]
    pub _abstract: Option<String>,
    /// Path to a second file containing the abstract in a different language.
    pub abstract2: Option<String>,

    /// Custom header includes
    #[structopt(long = "header-includes")]
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
    pub input_dir: PathBuf,
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
    pub author: Option<String>,
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
    pub _abstract: Option<PathBuf>,
    pub abstract2: Option<PathBuf>,

    pub header_includes: Vec<String>,

    // geometry
    pub geometry: Geometry,
}

impl Config {
    /// tempdir must live as long as Config
    pub fn new(args: CliArgs, infile: FileConfig, file: FileConfig, tempdir: &TempDir) -> Config {
        // verify input file
        match &args.input {
            FileOrStdio::StdIo => (),
            FileOrStdio::File(path) if path.is_file() => (),
            FileOrStdio::File(path) => panic!("Invalid File {:?}", path),
        }
        // cli > infile > configfile
        let output_type = match args.fileconfig.output_type.or(infile.output_type).or(file.output_type) {
            Some(typ) => typ,
            None => match &args.output {
                Some(FileOrStdio::StdIo) => OutType::Latex,
                Some(FileOrStdio::File(path)) => path.extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| {
                        (ext.eq_ignore_ascii_case("tex") || ext.eq_ignore_ascii_case("latex"))
                            .as_some(OutType::Latex)
                    })
                    .unwrap_or(OutType::Pdf),
                None => OutType::Pdf,
            }
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
                }
                FileOrStdio::File(filename)
            }
        };

        let input_dir = match &args.input {
            FileOrStdio::StdIo => env::current_dir()
                .expect("Can't use stdin without a current working directory"),
            FileOrStdio::File(file) => file.canonicalize()
                .expect("error canonicalising input file path")
                .parent().unwrap().to_owned(),
        };

        let bibliography = args.fileconfig.bibliography
            .or(infile.bibliography)
            .or(file.bibliography)
            .map(|bib| PathBuf::from(bib))
            .or_else(|| {
                if Path::new("references.bib").is_file() {
                    Some(PathBuf::from("references.bib"))
                } else {
                    None
                }
            });
        check_file_exists(&bibliography, "bibliography");
        let template = args.fileconfig.template
            .or(infile.template)
            .or(file.template)
            .map(PathBuf::from);
        check_file_exists(&template, "template");

        let lang = args.fileconfig.lang
            .or(infile.lang)
            .or(file.lang);
        let lang = match lang {
            None => Language::Eng,
            Some(lang) => Language::from_639_1(&lang)
                .or_else(|| Language::from_639_3(&lang))
                .or_else(|| Language::from_locale(&lang))
                // TODO: improve error message (with origin and value)
                .expect("Unknown language parameter")
        };

        let mut classoptions = HashSet::new();
        classoptions.extend(args.fileconfig.classoptions);
        classoptions.extend(infile.classoptions);
        classoptions.extend(file.classoptions);

        let mut header_includes = args.fileconfig.header_includes;
        header_includes.extend(infile.header_includes);
        header_includes.extend(file.header_includes);

        let citationstyle = args.fileconfig.citationstyle
            .or(infile.citationstyle)
            .or(file.citationstyle);

        let logo_university = args.fileconfig.logo_university
            .or(infile.logo_university)
            .or(file.logo_university)
            .map(PathBuf::from);
        check_file_exists(&logo_university, "logo-university");
        let logo_faculty = args.fileconfig.logo_faculty
            .or(infile.logo_faculty)
            .or(file.logo_faculty)
            .map(PathBuf::from);
        check_file_exists(&logo_faculty, "logo-faculty");
        let _abstract = args.fileconfig._abstract
            .or(infile._abstract)
            .or(file._abstract)
            .map(PathBuf::from);
        check_file_exists(&_abstract, "abstract");
        let abstract2 = args.fileconfig.abstract2
            .or(infile.abstract2)
            .or(file.abstract2)
            .map(PathBuf::from);
        check_file_exists(&abstract2, "abstract2");

        let document_type = args.fileconfig.document_type
                .or(infile.document_type)
                .or(file.document_type)
                .unwrap_or(DocumentType::Article);

        Config {
            output,
            out_dir: args.out_dir.unwrap_or(tempdir.path().to_owned()),
            temp_dir: tempdir.path().to_owned(),
            input: args.input,
            input_dir,
            output_type,
            document_type,
            bibliography,
            template,
            lang,
            citestyle: args.fileconfig.citestyle
                .or(infile.citestyle)
                .or(file.citestyle)
                .or_else(|| citationstyle.as_ref().cloned())
                .unwrap_or(MaybeUnknown::Known(CitationStyle::NumericComp)),
            bibstyle: args.fileconfig.bibstyle
                .or(infile.bibstyle)
                .or(file.bibstyle)
                .or(citationstyle)
                .unwrap_or(MaybeUnknown::Known(CitationStyle::Ieee)),
            figures: args.fileconfig.figures
                .or(infile.figures)
                .or(file.figures)
                .unwrap_or_else(|| match document_type {
                    DocumentType::Article => false,
                    DocumentType::Thesis => true,
                }),
            fontsize: args.fileconfig.fontsize
                .or(infile.fontsize)
                .or(file.fontsize)
                .unwrap_or_else(|| "11pt".to_string()),
            oneside: args.fileconfig.oneside
                .or(infile.oneside)
                .or(file.oneside)
                .unwrap_or(false),
            titlepage: args.fileconfig.titlepage
                .or(infile.titlepage)
                .or(file.titlepage)
                .unwrap_or(true),
            title: args.fileconfig.title
                .or(infile.title)
                .or(file.title),
            subtitle: args.fileconfig.subtitle
                .or(infile.subtitle)
                .or(file.subtitle),
            author: args.fileconfig.author
                .or(infile.author)
                .or(file.author),
            date: args.fileconfig.date
                .or(infile.date)
                .or(file.date),
            publisher: args.fileconfig.publisher
                .or(infile.publisher)
                .or(file.publisher),
            advisor: args.fileconfig.advisor
                .or(infile.advisor)
                .or(file.advisor),
            supervisor: args.fileconfig.supervisor
                .or(infile.supervisor)
                .or(file.supervisor),
            logo_university,
            logo_faculty,
            university: args.fileconfig.university
                .or(infile.university)
                .or(file.university),
            faculty: args.fileconfig.faculty
                .or(infile.faculty)
                .or(file.faculty),
            thesis_type: args.fileconfig.thesis_type
                .or(infile.thesis_type)
                .or(file.thesis_type),
            location: args.fileconfig.location
                .or(infile.location)
                .or(file.location),
            disclaimer: args.fileconfig.disclaimer
                .or(infile.disclaimer)
                .or(file.disclaimer),
            _abstract,
            abstract2,
            classoptions,
            header_includes,
            geometry: args.fileconfig.geometry
                .merge(infile.geometry)
                .merge(file.geometry),
        }
    }
}

fn check_file_exists<P: AsRef<Path>>(path: &Option<P>, cfgoption: &str) {
    if let Some(path) = path {
        if !path.as_ref().exists() {
            // TODO: better error handling
            panic!("{} file doesn't exist: {:?}", cfgoption, path.as_ref());
        }
        if !path.as_ref().is_file() {
            // TODO: better error handling
            panic!("{} file isn't a file", cfgoption);
        }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub fn to_read(&self) -> Box<Read> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdin())).lock()),
            FileOrStdio::File(path) => Box::new(File::open(path).expect("can't open input source")),
        }
    }

    pub fn to_write(&self) -> Box<Write> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdout())).lock()),
            FileOrStdio::File(path) => Box::new(File::create(path).expect("can't open output source")),
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
        } else {
            Err(format!("unknown output type {:?}", s))
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, EnumString)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum DocumentType {
    Article,
    Thesis,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumString)]
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
