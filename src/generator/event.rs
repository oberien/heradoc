use std::borrow::Cow;
use std::path::{PathBuf, Path};
use std::fmt;
use std::ffi::OsString;
use std::str::FromStr;

use librsvg::{Loader, LoadingError, RenderingError, CairoRenderer};
use cairo::{PdfSurface, Context, Rectangle};

pub use crate::frontend::{
    Tag,
    BiberReference,
    CodeBlock,
    Enumerate,
    Figure,
    FootnoteDefinition,
    FootnoteReference,
    Graphviz,
    Header,
    InterLink,
    MathBlock,
    MathBlockKind,
    Table,
    TaskListMarker,
    Url,
    Proof,
    ProofKind,
};
pub use pulldown_cmark::Alignment;

use crate::frontend::{Event as FeEvent, Size};
use crate::frontend::range::WithRange;
use crate::generator::Events;
use crate::resolve::Command;

// transformation of frontend::Event
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    Latex(Cow<'a, str>),
    IncludeMarkdown(Box<Events<'a>>),
    FootnoteReference(FootnoteReference<'a>),
    BiberReferences(Vec<BiberReference<'a>>),
    /// Url without content
    Url(Url<'a>),
    /// InterLink without content
    InterLink(InterLink<'a>),
    Image(Image<'a>),
    Svg(Svg<'a>),
    Label(Cow<'a, str>),
    Pdf(Pdf),
    SoftBreak,
    HardBreak,
    TaskListMarker(TaskListMarker),
    TableOfContents,
    Bibliography,
    ListOfTables,
    ListOfFigures,
    ListOfListings,
    Appendix,
}

/// Image to display as figure.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Image<'a> {
    pub label: Option<WithRange<Cow<'a, str>>>,
    pub caption: Option<WithRange<Cow<'a, str>>>,
    pub title: Option<Cow<'a, str>>,
    pub alt_text: Option<String>,
    /// Path to read image from.
    pub path: PathBuf,
    pub scale: Option<WithRange<Cow<'a, str>>>,
    pub width: Option<WithRange<Cow<'a, str>>>,
    pub height: Option<WithRange<Cow<'a, str>>>,
}

/// Vectorgraphic to display as figure.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Svg<'a> {
    pub label: Option<WithRange<Cow<'a, str>>>,
    pub caption: Option<WithRange<Cow<'a, str>>>,
    pub title: Option<Cow<'a, str>>,
    pub alt_text: Option<String>,
    /// Path to read image from.
    pub path: PathBuf,
    pub scale: Option<WithRange<Cow<'a, str>>>,
    pub width: Option<WithRange<Cow<'a, str>>>,
    pub height: Option<WithRange<Cow<'a, str>>>,
}

pub enum SvgConversionError {
    UnknownDimensions,
    LoadingError(LoadingError),
    RenderingError(RenderingError),
}

impl fmt::Display for SvgConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SvgConversionError::UnknownDimensions => write!(f, "unknown dimensions"),
            SvgConversionError::LoadingError(err) => write!(f, "can't load svg: {}", err),
            SvgConversionError::RenderingError(err) => write!(f, "conversion from svg to pdf failed: {}", err),
        }
    }
}
impl From<LoadingError> for SvgConversionError {
    fn from(err: LoadingError) -> Self {
        SvgConversionError::LoadingError(err)
    }
}
impl From<RenderingError> for SvgConversionError {
    fn from(err: RenderingError) -> Self {
        SvgConversionError::RenderingError(err)
    }
}

impl<'a> Svg<'a> {
    /// Converts the SVG to a PDF file and returns its path.
    ///
    /// This can be used by backends like latex, which don't support SVGs.
    pub fn to_pdf_path<P: AsRef<Path>>(&self, out_dir: P) -> Result<PathBuf, SvgConversionError> {
        let pdf_extension = self.path.extension()
            .map(|s| { let mut s = s.to_os_string(); s.push(".pdf"); s })
            .unwrap_or_else(|| OsString::from("pdf"));
        let mut pdf_path = out_dir.as_ref().join(self.path.file_name().unwrap());
        pdf_path.set_extension(pdf_extension);
        let handle = Loader::new().read_path(&self.path)?;
        let renderer = CairoRenderer::new(&handle);

        // cairo uses 72ppi by default, which is equal to 12pt
        let width = self.width.as_ref()
            .and_then(|width| Size::from_str(&width.0).ok())
            .and_then(|size| size.to_f64_opt(72.0, 12.0))
            .or_else(|| Size::from(renderer.intrinsic_dimensions().width?).to_f64_opt(72.0, 12.0))
            .or_else(|| renderer.intrinsic_dimensions().vbox.map(|vbox| vbox.width))
            .ok_or(SvgConversionError::UnknownDimensions)?;
        let height = self.height.as_ref()
            .and_then(|height| Size::from_str(&height.0).ok())
            .and_then(|size| size.to_f64_opt(72.0, 12.0))
            .or_else(|| Size::from(renderer.intrinsic_dimensions().height?).to_f64_opt(72.0, 12.0))
            .or_else(|| renderer.intrinsic_dimensions().vbox.map(|vbox| vbox.height))
            .ok_or(SvgConversionError::UnknownDimensions)?;
        let surface = PdfSurface::new(width, height, &pdf_path);
        let cr = Context::new(&surface);
        renderer.render_document(
            &cr,
            &Rectangle { x: 0.0, y: 0.0, width, height },
        )?;
        Ok(pdf_path)
    }
}

/// Pdf to include at that point inline.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Pdf {
    /// Path to read pdf from.
    pub path: PathBuf,
}

impl<'a> From<FeEvent<'a>> for Event<'a> {
    fn from(e: FeEvent<'a>) -> Self {
        match e {
            FeEvent::Start(tag) => Event::Start(tag),
            FeEvent::End(tag) => Event::End(tag),
            FeEvent::Text(text) => Event::Text(text),
            FeEvent::Html(html) => Event::Html(html),
            FeEvent::InlineHtml(html) => Event::InlineHtml(html),
            FeEvent::Latex(latex) => Event::Latex(latex),
            FeEvent::FootnoteReference(fnote) => Event::FootnoteReference(fnote),
            FeEvent::BiberReferences(biber) => Event::BiberReferences(biber),
            FeEvent::Url(url) => Event::Url(url),
            FeEvent::InterLink(interlink) => Event::InterLink(interlink),
            FeEvent::Include(_img) => unreachable!("Include is handled by Generator"),
            FeEvent::Label(label) => Event::Label(label),
            FeEvent::SoftBreak => Event::SoftBreak,
            FeEvent::HardBreak => Event::HardBreak,
            FeEvent::TaskListMarker(marker) => Event::TaskListMarker(marker),

            FeEvent::Command(command) => command.into(),
            FeEvent::ResolveInclude(_include) => {
                unreachable!("ResolveInclude is handled by Generator")
            },
        }
    }
}

impl<'a> From<Command> for Event<'a> {
    fn from(command: Command) -> Self {
        match command {
            Command::Toc => Event::TableOfContents,
            Command::Bibliography => Event::Bibliography,
            Command::ListOfTables => Event::ListOfTables,
            Command::ListOfFigures => Event::ListOfFigures,
            Command::ListOfListings => Event::ListOfListings,
            Command::Appendix => Event::Appendix,
        }
    }
}
