use super::*;
use std::fs::{self, File};
use diagnostic::{FileId, Span};

use tempdir::TempDir;
use url::Url;

use crate::error::Error;

macro_rules! assert_match {
    ($left:expr, $right:pat if $cond:expr) => ({
        let left_val = $left;
        match &left_val {
            $right if $cond => (),
            _ => {
                panic!(r#"assertion failed: `match left`
  left: `{:?}`,
 right: `{:?}`"#, left_val, stringify!($right))
            }
        }
    });
    ($left:expr, $right:pat) => ({
        assert_match!($left, $right if true)
    });
}

/// Creates the following filestructure in a tempdir for testing purposes.
/// project-root: heradoc-test-tempdir
/// document-root (only for testing): chapters
///
/// ```
/// heradoc-test-tempdir
/// ├── chapters
/// │   ├── chapter1.md
/// │   └── chapter2.md
/// ├── images
/// │   └── image.png
/// ├── image.png
/// ├── main.md
/// ├── pdf.pdf
/// └── test.md
/// ````
fn prepare() -> (TempDir, Span, Resolver, Diagnostics) {
    let tmpdir = TempDir::new("heradoc-test").expect("Can't create tempdir");
    let _ = File::create(tmpdir.path().join("main.md")).expect("Can't create main.md");
    let _ = File::create(tmpdir.path().join("test.md")).expect("Can't create test.md");
    let _ = File::create(tmpdir.path().join("image.png")).expect("Can't create image.png");
    let _ = File::create(tmpdir.path().join("pdf.pdf")).expect("Can't create pdf.pdf");
    fs::create_dir(tmpdir.path().join("chapters")).expect("Can't create chapter subdir");
    fs::create_dir(tmpdir.path().join("images")).expect("Can't create images subdir");
    let _ = File::create(tmpdir.path().join("chapters/chapter1.md")).expect("Can't create chapters/chapter1.md");
    let _ = File::create(tmpdir.path().join("chapters/chapter2.md")).expect("Can't create chapters/chapter2.md");
    let _ = File::create(tmpdir.path().join("images/image.png")).expect("Can't create images/image.png");
    fs::create_dir(tmpdir.path().join("downloads")).expect("can't create downloads directory");
    let span = Span { file: FileId::synthetic("test"), start: 0, end: 0 };
    let diagnostics = Diagnostics::new();
    diagnostics.add_synthetic_file("test", "".to_string());
    let resolver = Resolver::new(tmpdir.path().to_owned(), tmpdir.path().join("chapters"), tmpdir.path().join("download"));
    (tmpdir, span, resolver, diagnostics)
}

#[test]
fn relative_to_project_root() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();
    let ctx_chapters = Context::from_path("chapters").expect("can't create context");
    let ctx_chapters2 = Context::from_path("chapters/").expect("can't create context");

    let test = |ctx: Context| {
        let main = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/main.md", span, &diagnostics)
            .expect("failed to resolve `/main.md`");
        assert_match!(main, Include::Markdown(path, ctx) if path == &project_root.path().join("main.md") && ctx.url.as_str() == "heradoc://project/main.md");

        let test = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/test.md", span, &diagnostics)
            .expect("failed to resolve `/test.md`");
        assert_match!(test, Include::Markdown(path, ctx) if path == &project_root.path().join("test.md") && ctx.url.as_str() == "heradoc://project/test.md");

        let image = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/image.png", span, &diagnostics)
            .expect("failed to resolve `/image.png`");
        assert_match!(image, Include::Image(path) if path == &project_root.path().join("image.png"));

        let pdf = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/pdf.pdf", span, &diagnostics)
            .expect("failed to resolve `/pdf.pdf`");
        assert_match!(pdf, Include::Pdf(path) if path == &project_root.path().join("pdf.pdf"));

        let chapter1 = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/chapters/chapter1.md", span, &diagnostics)
            .expect("failed to resolve `/chapters/chapter1.md`");
        assert_match!(chapter1, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter1.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter1.md");

        let chapter2 = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/chapters/chapter2.md", span, &diagnostics)
            .expect("failed to resolve `/chapters/chapter2.md`");
        assert_match!(chapter2, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter2.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter2.md");

        let image = resolver
            .resolve(ResolveSecurity::Default, &ctx, "/images/image.png", span, &diagnostics)
            .expect("failed to resolve `/images/image.png`");
        assert_match!(image, Include::Image(path) if path == &project_root.path().join("images/image.png"));
    };

    test(ctx);
    test(ctx_chapters);
    test(ctx_chapters2);
}

#[test]
fn relative_to_document_root() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();
    let ctx_chapters = Context::from_path("chapters").expect("can't create context");
    let ctx_images = Context::from_path("images").expect("can't create context");

    let test = |ctx: Context| {
        let chapter1 = resolver
            .resolve(ResolveSecurity::Default, &ctx, "//document/chapter1.md", span, &diagnostics)
            .expect("failed to resolve `//document/chapter1.md`");
        assert_match!(chapter1, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter1.md") && ctx.url.as_str() == "heradoc://document/chapter1.md");

        let chapter2 = resolver
            .resolve(ResolveSecurity::Default, &ctx, "//document/chapter2.md", span, &diagnostics)
            .expect("failed to resolve `//document/chapter2.md`");
        assert_match!(chapter2, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter2.md") && ctx.url.as_str() == "heradoc://document/chapter2.md");
    };

    test(ctx);
    test(ctx_chapters);
    test(ctx_images);
}

#[test]
fn relative_to_current_file() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();
    let ctx_chapters = Context::from_path("chapters/").expect("can't create context");

    let main = resolver
        .resolve(ResolveSecurity::Default, &ctx, "main.md", span, &diagnostics)
        .expect("failed to resolve `main.md`");
    let main2 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "../main.md", span, &diagnostics)
        .expect("failed to resolve `../main.md`");
    assert_match!(main, Include::Markdown(path, ctx) if path == &project_root.path().join("main.md") && ctx.url.as_str() == "heradoc://project/main.md");
    assert_match!(main2, Include::Markdown(path, ctx) if path == &project_root.path().join("main.md") && ctx.url.as_str() == "heradoc://project/main.md");

    let test = resolver
        .resolve(ResolveSecurity::Default, &ctx, "test.md", span, &diagnostics)
        .expect("failed to resolve `test.md`");
    let test2 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "../test.md", span, &diagnostics)
        .expect("failed to resolve `test.md`");
    assert_match!(test, Include::Markdown(path, ctx) if path == &project_root.path().join("test.md") && ctx.url.as_str() == "heradoc://project/test.md");
    assert_match!(test2, Include::Markdown(path, ctx) if path == &project_root.path().join("test.md") && ctx.url.as_str() == "heradoc://project/test.md");

    let image = resolver
        .resolve(ResolveSecurity::Default, &ctx, "image.png", span, &diagnostics)
        .expect("failed to resolve `image.png`");
    let image2 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "../image.png", span, &diagnostics)
        .expect("failed to resolve `../image.png`");
    assert_match!(image, Include::Image(path) if path == &project_root.path().join("image.png"));
    assert_match!(image2, Include::Image(path) if path == &project_root.path().join("image.png"));

    let pdf = resolver
        .resolve(ResolveSecurity::Default, &ctx, "pdf.pdf", span, &diagnostics)
        .expect("failed to resolve `pdf.pdf`");
    let pdf2 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "../pdf.pdf", span, &diagnostics)
        .expect("failed to resolve `../pdf.pdf`");
    assert_match!(pdf, Include::Pdf(path) if path == &project_root.path().join("pdf.pdf"));
    assert_match!(pdf2, Include::Pdf(path) if path == &project_root.path().join("pdf.pdf"));

    let chapter1 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "chapters/chapter1.md", span, &diagnostics)
        .expect("failed to resolve `chapters/chapter1.md`");
    let chapter12 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "chapter1.md", span, &diagnostics)
        .expect("failed to resolve `chapter1.md`");
    assert_match!(chapter1, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter1.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter1.md");
    assert_match!(chapter12, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter1.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter1.md");

    let chapter2 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "chapters/chapter2.md", span, &diagnostics)
        .expect("failed to resolve `chapters/chapter2.md`");
    let chapter22 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "chapter2.md", span, &diagnostics)
        .expect("failed to resolve `chapter2.md`");
    assert_match!(chapter2, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter2.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter2.md");
    assert_match!(chapter22, Include::Markdown(path, ctx) if path == &project_root.path().join("chapters/chapter2.md") && ctx.url.as_str() == "heradoc://project/chapters/chapter2.md");

    let image = resolver
        .resolve(ResolveSecurity::Default, &ctx, "images/image.png", span, &diagnostics)
        .expect("failed to resolve `images/image.png`");
    let image2 = resolver
        .resolve(ResolveSecurity::Default, &ctx_chapters, "../images/image.png", span, &diagnostics)
        .expect("failed to resolve `../images/image.png`");
    assert_match!(image, Include::Image(path) if path == &project_root.path().join("images/image.png"));
    assert_match!(image2, Include::Image(path) if path == &project_root.path().join("images/image.png"));
}

#[test]
fn commands() {
    let (_project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();
    let ctx2 = Context::from_path("chapters/").expect("can't create context");

    // if one works from a subdir, all work from a a subdir
    let toc = resolver
        .resolve(ResolveSecurity::Default, &ctx2, "//toc", span, &diagnostics)
        .expect("failed to resolve `//toc`");
    assert_match!(toc, Include::Command(Command::Toc));

    // test all commands
    let appendix = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//appendix", span, &diagnostics)
        .expect("failed to resolve `//appendix`");
    assert_match!(appendix, Include::Command(Command::Appendix));
    let toc = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//toc", span, &diagnostics)
        .expect("failed to resolve `//toc`");
    let toc2 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//TOC", span, &diagnostics)
        .expect("failed to resolve `//TOC`");
    let toc3 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//tableofcontents", span, &diagnostics)
        .expect("failed to resolve `//tableofcontents`");
    assert_match!(toc, Include::Command(Command::Toc));
    assert_match!(toc2, Include::Command(Command::Toc));
    assert_match!(toc3, Include::Command(Command::Toc));
    let bib = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//bibliography", span, &diagnostics)
        .expect("failed to resolve `//bibliography`");
    let bib2 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//references", span, &diagnostics)
        .expect("failed to resolve `//references`");
    assert_match!(bib, Include::Command(Command::Bibliography));
    assert_match!(bib2, Include::Command(Command::Bibliography));
    let tables = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//listoftables", span, &diagnostics)
        .expect("failed to resolve `//listoftables`");
    assert_match!(tables, Include::Command(Command::ListOfTables));
    let figures = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//listoffigures", span, &diagnostics)
        .expect("failed to resolve `//listoffigures`");
    assert_match!(figures, Include::Command(Command::ListOfFigures));
    let listings = resolver
        .resolve(ResolveSecurity::Default, &ctx, "//listoflistings", span, &diagnostics)
        .expect("failed to resolve `//listoflistings`");
    assert_match!(listings, Include::Command(Command::ListOfListings));
}

#[test]
fn http_resolves_needs_internet() {
    let (_project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();

    let external = resolver
        .resolve(ResolveSecurity::Default,
            &ctx,
            "https://raw.githubusercontent.com/oberien/heradoc/master/README.md",
            span,
            &diagnostics,
        ).expect("failed to download external document");
    assert_match!(external, Include::Markdown(_, ctx) if ctx.typ() == ContextType::Remote);
}

#[test]
fn local_resolves_not_exist_not_internal_bug() {
    let (_project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();

    let error = resolver
        .resolve(ResolveSecurity::Default, &ctx, "this_file_does_not_exist.md", span, &diagnostics)
        .expect_err("only files that exist on disk can be resolved");
    assert_match!(error, Error::Diagnostic);
}

#[test]
fn local_absolute_url_stays_absolute() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();

    let url = Url::from_file_path(project_root.path().join("main.md")).unwrap();
    let main = resolver
        .resolve(ResolveSecurity::Default, &ctx, url.as_str(), span, &diagnostics)
        .expect("failed to resolve absolute file url");

    assert_match!(main, Include::Markdown(_, ctx) if ctx.typ() == ContextType::LocalAbsolute);
}

#[test]
fn url_does_not_exist() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_project_root();

    let url = Url::from_file_path(project_root.path().join("this_file_does_not_exist.md")).unwrap();
    let error = resolver
        .resolve(ResolveSecurity::Default, &ctx, url.as_str(), span, &diagnostics)
        .expect_err("failed to resolve absolute file url");

    assert_match!(error, Error::Diagnostic);
}

#[test]
fn relative_dot_slash_in_subdirectory() {
    let (project_root, span, resolver, diagnostics) = prepare();
    let ctx = Context::from_path("chapters/").expect("can't create context");

    let chapter1 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "chapter1.md", span, &diagnostics)
        .expect("failed to resolve sibling file");
    let chapter12 = resolver
        .resolve(ResolveSecurity::Default, &ctx, "./chapter1.md", span, &diagnostics)
        .expect("failed to resolve sibling file via explicitely relative path");

    assert_eq!(chapter1, chapter12);
    assert_match!(chapter1, Include::Markdown(path, ctx)
                        if path == &project_root.path().join("chapters/chapter1.md")
                            && ctx.typ() == ContextType::LocalRelative);
}
