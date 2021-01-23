use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs::File;
use std::fmt::Write as _;
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
    /// Initially gather crate state, and put root item on stack.
    Root,
    /// Traverse into an item, dispatching on its kind.
    Item(types::Id),
}

impl<'a> Iterator for Rustdoc<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(buffered) = self.appender.buffered.pop_front() {
                return Some(buffered);
            }

            if let Some(traverse) = self.appender.stack.pop() {
                self.traverse(traverse);
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
                    .current_dir(&path)
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
                let krate = meta.workspace_members[0].split(' ').next().unwrap();
                target.push(format!("{}.json", krate));

                let file = match File::open(&target) {
                    Ok(file) => file,
                    Err(err) => {
                        diag
                            .error("Failed to open rustdoc output data")
                            .note(target.display().to_string())
                            .emit();
                        return Err(Fatal::Output(err));
                    }
                };

                match serde_json::from_reader(file) {
                    Ok(krate) => Ok(krate),
                    Err(err) => {
                        diag
                            .error("Cargo metadata failed for crate")
                            .note(String::from_utf8_lossy(&format.stderr))
                            .emit();
                        Err(Fatal::Output(err.into()))
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

impl<'a> Rustdoc<'a> {
    /// Get the next item while traversing a particular item.
    /// This will also push more items or a remaining tail to its stack.
    fn traverse(&mut self, what: Traversal) {
        match what {
            Traversal::Root => self.appender.root(&self.krate),
            Traversal::Item(id) => self.append_item_by_id(&id),
        }
    }

    fn append_item_by_id(&mut self, id: &types::Id) {
        if let Some(item) = self.krate.index.get(id) {
            let krate = &self.krate;
            match item {
                Item { inner: ItemEnum::ModuleItem(inner), .. } => {
                    self.appender.module(krate, item, inner);
                },
                Item { inner: ItemEnum::StructItem(inner), .. } => {
                    self.appender.struct_(krate, item, inner);
                },
                _ => eprintln!("Unimplemented {:?}", item),
            }
        } else {
            self.invalid_item(Traversal::Item(id.clone()));
        }
    }

    /// Invoked when we encounter an unexpected item/reference.
    fn invalid_item(&mut self, what: Traversal) {
        let mut builder = self.diagnostics
            .bug("Unexpected item in rustdoc json output")
            .note(format!("Traversing {:?}", what));

        if let Traversal::Item(id) = what {
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
    fn root(&mut self, krate: &types::Crate) {
        let label = self.label_for_id(&krate.root, krate).unwrap();
        let header = frontend::Header {
            label: WithRange(Cow::Owned(label), (0..0).into()),
            level: 1,
        };

        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            let root = krate.paths.get(&krate.root).unwrap();
            let lib_name = root.path[0].clone();
            Cow::Owned(lib_name)
        }));
        self.buffered.push_back(Event::End(Tag::Header(header)));

        if krate.includes_private {
            self.buffered.push_back(Event::Text(Cow::Borrowed(
                "Note: This development documentation includes private items which are not accessible from the outside.",
            )));
        }

        self.stack.push(Traversal::Item(krate.root.clone()));
    }

    // Handle the individual items.
    // Each methods types the crate environment, the full item, and its specialized enum internals.

    fn module(&mut self, krate: &types::Crate, item: &Item, module: &types::Module) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        let label = self.label_for_item_at_path(&summary.path);

        let header = frontend::Header {
            label: WithRange(Cow::Owned(label), (0..0).into()),
            level: 2,
        };

        // Add a header.
        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            let qualifier = if module.is_crate { "Crate" } else { "Module" };
            let meta = match &item.visibility {
                types::Visibility::Public => "pub ".to_string(),
                types::Visibility::Default => "".to_string(),
                types::Visibility::Crate => "pub(crate) ".to_string(),
                types::Visibility::Restricted  { parent: _, path } => {
                    format!("pub({}) ", path)
                },
            };
            let module_name = self.name_for_item_at_path(&summary.path);
            Cow::Owned(format!("{} {}{}", qualifier, meta, module_name))
        }));
        self.buffered.push_back(Event::End(Tag::Header(header)));

        // Describe all children in text.
        self.buffered.push_back(Event::Start(Tag::List));
        for child in &module.items {
            self.buffered.push_back(Event::Start(Tag::Item));
            if let Some(target) = krate.paths.get(child) {
                let child_label = self.label_for_item_at_path(&target.path);

                let link = frontend::InterLink {
                    label: Cow::Owned(child_label),
                    uppercase: false,
                };

                self.buffered.push_back(Event::Start(Tag::InterLink(link.clone())));
                self.buffered.push_back(Event::Text({
                    let item_name = self.name_for_item_at_path(&target.path);
                    Cow::Owned(item_name)
                }));
                self.buffered.push_back(Event::End(Tag::InterLink(link)));
            } else if let Some(Item { name: Some(name), .. }) = krate.index.get(child) {
                self.buffered.push_back(Event::Text(name.clone().into()));
            } else if let Some(item) = krate.index.get(child) {
                eprintln!("Encountered weird module child: {:?}", item);
            } else {
                eprintln!("Encountered weird module child with no item: {:?}", child);
            }

            self.buffered.push_back(Event::End(Tag::Item));
        }
        self.buffered.push_back(Event::End(Tag::List));

        // And queue to dispatch into children.
        for child in module.items.iter().rev() {
            self.stack.push(Traversal::Item(child.clone()))
        }
    }

    fn struct_(&mut self, krate: &types::Crate, item: &Item, struct_: &types::Struct) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        let label = self.label_for_item_at_path(&summary.path);

        // Avoid allocating too much below..
        if struct_.fields.len() >= 1_000_000 {
            panic!("Number of fields too large, considering opening a pull request to turn this into an iterative procedure.");
        }

        let header = frontend::Header {
            label: WithRange(Cow::Owned(label.clone()), (0..0).into()),
            level: 3,
        };

        let meta = Self::codify_visibility(&item.visibility);
        let struct_name = self.name_for_item_at_path(&summary.path);
        let mut def = item.attrs.join("\n");
        writeln!(&mut def, "{}struct {} {{", meta, struct_name)
            .expect("Writing to string succeeds");

        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            Cow::Owned(format!("Struct {}{}", meta, struct_name))
        }));
        // TODO: generics?
        self.buffered.push_back(Event::End(Tag::Header(header.clone())));

        let mut field_documentation = vec![];
        for field_id in &struct_.fields {
            if let Some(Item {
                inner: ItemEnum::StructFieldItem(field),
                name: Some(name),
                visibility,
                docs,
                ..
            }) = krate.index.get(field_id) {
                let meta = Self::codify_visibility(visibility);
                let type_name = Self::codify_type(krate, field);
                writeln!(&mut def, "    {}{}: {}", meta, name, type_name)
                    .expect("Writing to string succeeds");
                field_documentation.push((name, field, type_name, docs));
            } else {
                // FIXME: should not occur.
            }
        }

        if struct_.fields_stripped {
            def.push_str("    // some fields omitted\n");
        }
        def.push('}');

        let def_block_tag = frontend::CodeBlock {
            label: None,
            caption: None,
            language: Some(WithRange(Cow::Borrowed("rust"), (0..0).into())),
        };
        self.buffered.push_back(Event::Start(Tag::CodeBlock(def_block_tag.clone())));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(def_block_tag.clone())));

        // FIXME: we would like a level-4 header..
        if !field_documentation.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineEmphasis));
            self.buffered.push_back(Event::Text(Cow::Borrowed("Fields")));
            self.buffered.push_back(Event::End(Tag::InlineEmphasis));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        for (name, _field_type, type_name, docs) in field_documentation {
            self.buffered.push_back(Event::Start(Tag::Paragraph));

            self.buffered.push_back(Event::Start(Tag::InlineCode));
            self.buffered.push_back(Event::Text(Cow::Owned(name.clone())));
            self.buffered.push_back(Event::Text(Cow::Borrowed(": ")));
            // FIXME: link to the type, if appropriate.
            // self.buffered.push_back(Event::Start(Tag::InterLink(field_type_link.clone())));
            self.buffered.push_back(Event::Text(Cow::Owned(type_name.clone())));
            // self.buffered.push_back(Event::End(Tag::InterLink(field_type_link.clone())));
            self.buffered.push_back(Event::End(Tag::InlineCode));

            self.buffered.push_back(Event::Text(Cow::Borrowed("  ")));
            // FIXME: treat as recursive markdown?
            self.buffered.push_back(Event::Text(Cow::Owned(docs.clone())));

            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        if !item.docs.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Text(item.docs.clone().into()));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }
    }

    fn label_for_id(&self, path: &types::Id, krate: &types::Crate) -> Option<String> {
        match krate.paths.get(path) {
            Some(summary) => Some(self.label_for_item_at_path(&summary.path)),
            None => None,
        }
    }

    fn codify_visibility(visibility: &types::Visibility) -> String {
        match visibility {
            types::Visibility::Public => "pub ".to_string(),
            types::Visibility::Default => "".to_string(),
            types::Visibility::Crate => "pub(crate) ".to_string(),
            types::Visibility::Restricted  { parent: _, path } => {
                format!("pub({}) ", path)
            },
        }
    }

    fn codify_type(krate: &types::Crate, type_: &types::Type) -> String {
        #[allow(clippy::enum_glob_use)]
        use types::Type::*;
        match type_ {
            ResolvedPath { name, args, param_names, .. } => {
                let name = name.clone();
                match args.as_ref().map(|a| &**a) {
                    None => {},
                    Some(types::GenericArgs::AngleBracketed { args, bindings }) => {
                        // Wait, do we need to map TypeBinding to args via names?
                        // FIXME: handle them, important for showing structs.
                        // todo!("Unhandled generic arguments to type");
                    }
                    Some(types::GenericArgs::Parenthesized { .. }) => {
                        // FIXME: handle as error, probably?
                        todo!("Can this occur?");
                    }
                }
                name
            },
            Generic(st) | Primitive(st) => st.clone(),
            Tuple(items) => {
                let mut items = items.iter();
                let first = match items.next() {
                    None => return "()".into(),
                    Some(first) => first,
                };
                let mut name = format!("({}", Self::codify_type(krate, first));
                for type_ in items {
                    name.push(',');
                    name.push_str(&Self::codify_type(krate, type_));
                }
                name.push(')');
                name
            },
            Slice(inner) => format!("[{}]", Self::codify_type(krate, inner)),
            Array { type_, len } => {
                format!("[{}; {}]", Self::codify_type(krate, type_), len)
            },
            // ImplTrait..
            Never => "!".into(),
            Infer => "_".into(),
            RawPointer { mutable, type_ } => {
                let qualifier = if *mutable { "mut" } else { "const" };
                format!("*{} {}", qualifier, Self::codify_type(krate, type_))
            }
            BorrowedRef { lifetime, mutable, type_ } => {
                let lifetime = lifetime.as_ref().map_or("", |st| st.as_str());
                let qualifier = if *mutable { "mut " } else { "" };
                let type_ = Self::codify_type(krate, type_);
                format!("&{}{}{}", lifetime, qualifier, type_)
            }
            QualifiedPath { name, self_type, trait_ } => {
                let self_type = Self::codify_type(krate, self_type);
                let trait_ = Self::codify_type(krate, trait_);
                format!("<{} as {}>::{}", self_type, trait_, name)
            }
            // FIXME: where can we test this best?
            ImplTrait(_) | FunctionPointer(_) => todo!("Not yet implemented kind of named type encountered"),
        }
    }

    fn name_for_item_at_path(&self, path: &[String]) -> String {
        path.join("::")
    }

    fn label_for_item_at_path(&self, path: &[String]) -> String {
        path.join("-")
    }
}
