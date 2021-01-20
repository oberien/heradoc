use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs::File;
use std::sync::Arc;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::error::{Fatal, FatalResult};
use crate::frontend::{self, Event, Tag};
use crate::frontend::range::WithRange;

/// Contains the type definitions, all implementing Deserialize.
mod types;

use types::{Item, ItemEnum};

#[derive(Debug)]
pub struct Rustdoc<'a> {
    cfg: &'a Config,
    diagnostics: Arc<Diagnostics<'a>>,
    krate: types::Crate,
    /// The dynamic state. Implements the actual traversal methods, taking references to the crate
    /// data.
    appender: RustdocAppender<'a>,
}

#[derive(Debug)]
struct RustdocAppender<'a> {
    /// Stack of started, but not yet finished, portions of the documentation.
    stack: Vec<Traversal>,
    /// A buffer of events, yielded before continuing with the stack.
    buffered: VecDeque<Event<'a>>,
}

pub enum Crate {
    Local(PathBuf),
}

/// Denotes some part of the crate which we have not yet fully documented.
#[derive(Debug)]
enum Traversal {
    Root,
    Module(types::Id),
}

impl<'a> Iterator for Rustdoc<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(buffered) = self.appender.buffered.pop_front() {
                return Some(buffered);
            }

            if let Some(traverse) = self.appender.stack.pop() {
                if let Some(item) = self.traverse(traverse) {
                    return Some(item);
                }
            } else {
                return None;
            }
        }
    }
}

impl Crate {
    /// Invoke rustdoc to generate the json for this target.
    pub fn generate(&self, diag: &Diagnostics<'_>) -> FatalResult<types::Crate> {
        match self {
            Crate::Local(path) => {
                let metadata = Command::new("cargo")
                    .args(&["metadata", "--format-version", "1"])
                    .output()?;

                if !metadata.status.success() {
                    diag
                        .error("Cargo metadata failed for crate")
                        .note(String::from_utf8_lossy(&metadata.stderr))
                        .emit();
                    return Err(Fatal::Output(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Metadata call failed",
                    )));
                }

                let meta: types::WorkspaceMetadata = match serde_json::from_slice(&metadata.stdout) {
                    Ok(meta) => meta,
                    Err(err) => {
                        diag
                            .error("Failed to parse cargo metadata")
                            .emit();
                        return Err(Fatal::Output(err.into()));
                    }
                };

                let format = Command::new("cargo")
                    .args(&["+nightly", "rustdoc", "--", "--output-format", "json"])
                    .current_dir(&path)
                    .output()?;

                let mut target = PathBuf::from(meta.target_directory);
                target.push("doc");
                target.push(format!("{}.json", meta.packages[0].name));

                let file = File::open(target)?;

                match serde_json::from_reader(file) {
                    Ok(krate) => Ok(krate),
                    Err(err) => {
                        diag
                            .error("Cargo metadata failed for crate")
                            .note(String::from_utf8_lossy(&format.stderr))
                            .emit();
                        return Err(Fatal::Output(err.into()));
                    }
                }
            }
        }
    }
}

impl<'a> Rustdoc<'a> {
    pub fn new(cfg: &'a Config, krate: types::Crate, diagnostics: Arc<Diagnostics<'a>>) -> Rustdoc<'a> {
        Rustdoc {
            cfg,
            diagnostics,
            krate,
            appender: RustdocAppender::default(),
        }
    }
}

macro_rules! handle_item_by_id {
    (match ($self:ident, $what:expr) {
        $($variant:ident (id) as ItemEnum::$kind:ident => $handler:ident),*
    }) => {
        let what = $what;
        match &what {
            $(
                Traversal::$variant(id) => {
                    if let Some(Item { inner: ItemEnum::$kind(inner), .. })
                        = $self.krate.index.get(id)
                    {
                        let item = $self.krate.index.get(id).unwrap();
                        return $self.appender.$handler(item, inner);
                    } else {
                        $self.invalid_item(what);
                        return None;
                    }
                },
            ),*
            _ => {},
        };
    };
}

impl<'a> Rustdoc<'a> {
    /// Get the next item while traversing a particular item.
    /// This will also push more items or a remaining tail to its stack.
    fn traverse(&mut self, what: Traversal) -> Option<Event<'a>> {
        if let Traversal::Root = &what {
            return self.appender.root(&self.krate);
        }

        // FIXME: use @ before subpatterns (https://github.com/rust-lang/rust/issues/65490),
        // instead of macro
        handle_item_by_id!(match (self, what) {
            Module(id) as ItemEnum::ModuleItem => module
        });

        // unhandled kind of item, probably.
        None
    }

    /// Invoked when we encounter an unexpected item/reference.
    fn invalid_item(&mut self, what: Traversal) {
        let mut builder = self.diagnostics
            .bug("Unexpected item in rustdoc json output")
            .note(format!("Traversing {:?}", what));

        if let Traversal::Module(id) = what  {
            if let Some(item) = self.krate.index.get(&id) {
                if let Some(name) = &item.name {
                    builder = builder.note(name);
                }

                builder = builder.note(format!("Source Span {:?}", item.source));
            }
        }

        builder.emit();
    }
}

impl Default for RustdocAppender<'_> {
    fn default() -> Self {
        RustdocAppender {
            stack: vec![Traversal::Root],
            buffered: VecDeque::new(),
        }
    }
}

impl<'a> RustdocAppender<'a> {
    fn root(&mut self, krate: &types::Crate) -> Option<Event<'a>> {
        let label = self.label_for_id(&krate.root, krate).unwrap();
        let header = frontend::Header {
            label: WithRange(Cow::Owned(label), (0..0).into()),
            level: 0,
        };

        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            let root = krate.paths.get(&krate.root).unwrap();
            let lib_name = root.path[0].clone();
            Cow::Owned(lib_name)
        }));
        self.buffered.push_back(Event::End(Tag::Header(header)));

        self.stack.push(Traversal::Module(krate.root.clone()));
        None
    }

    fn module(&mut self, item: &Item, module: &types::Module) -> Option<Event<'a>> {
        None
    }

    fn label_for_id(&self, path: &types::Id, krate: &types::Crate) -> Option<String> {
        match krate.paths.get(path) {
            Some(summary) => Some(self.label_for_item_at_path(&summary.path)),
            None => None,
        }
    }

    fn label_for_item_at_path(&self, path: &[String]) -> String {
        path.join("-")
    }
}
