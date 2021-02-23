use std::borrow::Cow;
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::fmt::Write as _;
use std::sync::Arc;
use std::path::PathBuf;
use std::process::Command;

use itertools::Itertools as _;

use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::error::{Fatal, FatalResult};
use crate::frontend::{self, Event, Include, Tag};
use crate::frontend::range::WithRange;
use crate::resolve::{Context, Include as ResolvedInclude, ResolveSecurity};

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
    context: Context,
    simulated_source_path: PathBuf,
    /// Stack of started, but not yet finished, portions of the documentation.
    stack: Vec<Traversal>,
    /// A buffer of events, yielded before continuing with the stack.
    buffered: VecDeque<Event<'a>>,
    diagnostics: Arc<Diagnostics<'a>>,
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

                let mut target = PathBuf::from(meta.target_directory);
                target.push("doc");
                let krate = meta.workspace_members[0].split(' ').next().unwrap();

                let format = Command::new("cargo")
                    .args(&["+nightly", "rustdoc", "-p"])
                    .arg(&krate)
                    .args(&["--", "--output-format", "json"])
                    .current_dir(&path)
                    .output()?;

                if !format.status.success() {
                    diag
                        .error("Compiling rustdoc failed")
                        .note(String::from_utf8_lossy(&metadata.stdout))
                        .note(String::from_utf8_lossy(&metadata.stderr))
                        .note(format!("Compiling `{}` in `{}`", krate, path.display()))
                        .emit();
                    return Err(Fatal::Output(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Metadata call failed",
                    )));
                }

                target.push({
                    // FIXME: support actually renamed library targets?
                    let lib_name = format!("{}.json", krate);
                    lib_name.replace("-", "_")
                });

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
                            .note(err.to_string())
                            .emit();
                        Err(Fatal::Output(err.into()))
                    }
                }
            }
        }
    }
}

impl<'a> Rustdoc<'a> {
    pub fn new(
        cfg: &'a Config,
        krate: types::Crate,
        diagnostics: Arc<Diagnostics<'a>>,
        context: Context,
    ) -> Rustdoc<'a> {
        Rustdoc {
            cfg,
            diagnostics: Arc::clone(&diagnostics),
            krate,
            appender: RustdocAppender::new(cfg, diagnostics, context),
        }
    }

    pub fn context(&self) -> &Context {
        &self.appender.context
    }

    pub fn diagnostics(&self) -> Arc<Diagnostics<'a>> {
        self.diagnostics.clone()
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
                Item { inner: ItemEnum::EnumItem(inner), .. } => {
                    self.appender.enum_(krate, item, inner);
                },
                Item { inner: ItemEnum::ConstantItem(inner), .. } => {
                    self.appender.constant(krate, item, inner);
                },
                Item { inner: ItemEnum::StaticItem(inner), .. } => {
                    self.appender.static_(krate, item, inner);
                },
                Item { inner: ItemEnum::FunctionItem(inner), .. } => {
                    self.appender.function(krate, item, inner);
                },
                Item { inner: ItemEnum::TraitItem(inner), .. } => {
                    self.appender.trait_(krate, item, inner);
                },
                Item { inner: ItemEnum::ImplItem(inner), .. } => {
                    self.appender.impl_(krate, item, inner);
                },
                Item { inner: ItemEnum::TypedefItem(inner), .. } => {
                    self.appender.typedef(krate, item, inner);
                },
                Item { inner: ItemEnum::ImportItem(inner), .. } => {
                    self.appender.import(krate, item, inner);
                },
                Item { inner: ItemEnum::ExternCrateItem { name, rename }, .. } => {
                    self.appender.extern_item(krate, item, name, rename.as_ref());
                },
                Item { kind: types::ItemKind::Primitive, .. }
                | Item { kind: types::ItemKind::Keyword, .. } => {},
                _ => eprintln!("Unimplemented {:?}", item),
            }
        } else {
            self.invalid_item(Traversal::Item(id.clone()));
        }
    }

    /// Invoked when we encounter an unexpected item/reference.
    fn invalid_item(&mut self, what: Traversal) {
        let mut builder = self.appender.diagnostics
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

impl<'a> RustdocAppender<'a> {
    fn new(
        cfg: &'a Config,
        diagnostics: Arc<Diagnostics<'a>>,
        context: Context,
    ) -> Self {
        RustdocAppender {
            context,
            simulated_source_path: cfg.temp_dir.clone(),
            stack: vec![Traversal::Root],
            buffered: VecDeque::new(),
            diagnostics,
        }
    }

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
            let meta = Self::codify_visibility(&item.visibility);
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
        let name = item.name
            .as_ref()
            .expect("Struct without a name");

        // Avoid allocating too much below..
        if struct_.fields.len() >= 1_000_000 {
            panic!("Number of fields too large, considering opening a pull request to turn this into an iterative procedure.");
        }

        let meta = Self::codify_visibility(&item.visibility);

        let mut def: String = item.attrs
            .iter()
            .map(String::as_str)
            .interleave_shortest(std::iter::repeat("\n"))
            .collect();

        let (header_title, title) = match item.kind {
            types::ItemKind::Union => ("Union", "union"),
            types::ItemKind::Struct => ("Struct", "struct"),
            _ => unreachable!("Unexpected struct kind"),
        };

        self.append_header_for_inner_item(header_title, item, summary);

        write!(&mut def, "{}{} {}", meta, title, name)
            .expect("Writing to string succeeds");
        def.push_str(&self.codify_generics(krate, &struct_.generics));
        let (start_tag, end_tag) = match struct_.struct_type {
            types::StructType::Plain => ("{\n", "}"),
            types::StructType::Tuple => ("(\n", ")"),
            types::StructType::Unit => ("", ";"),
        };

        def.push_str(start_tag);
        let mut field_documentation = vec![];
        for field_id in &struct_.fields {
            match krate.index.get(field_id) {
                Some(Item {
                    inner: ItemEnum::StructFieldItem(field),
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let type_name = self.codify_type(krate, field);
                    def.push_str("    ");
                    def.push_str(&meta);
                    if let types::StructType::Tuple = struct_.struct_type {} else {
                        def.push_str(name);
                        def.push_str(": ");
                    }
                    def.push_str(&type_name);
                    def.push_str(",\n");
                    field_documentation.push((name, field, type_name, docs));
                }
                Some(other) => {
                    self.diagnostics
                        .warning(format!("Unhandled variant item: {:?}", other))
                        .note(format!("In enum {}", self.name_for_item_at_path(&summary.path)))
                        .emit();
                }
                None => unreachable!("Enum item does not exist?"),
            }
        }

        if struct_.fields_stripped {
            def.push_str("    // some fields omitted\n");
        }
        def.push_str(end_tag);

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);

        // FIXME: we would like a level-4 header..
        if !field_documentation.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineStrong));
            self.buffered.push_back(Event::Text(Cow::Borrowed("Fields")));
            self.buffered.push_back(Event::End(Tag::InlineStrong));
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

            if let Some(docs) = docs {
                self.buffered.push_back(Event::Text(Cow::Borrowed("  ")));
                // FIXME: treat as recursive markdown?
                self.append_docs(docs.clone());
            }

            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        self.append_impls(krate, item, &struct_.impls);
    }

    fn constant(&mut self, krate: &types::Crate, item: &Item, constant: &types::Constant) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        self.append_header_for_inner_item("Constant", item, summary);

        let meta = Self::codify_visibility(&item.visibility);
        let mut def = format!("{}const ", meta);
        match &item.name {
            Some(name) => def.push_str(name),
            // FIXME: error handling.
            _ => panic!("Const without a name"),
        }
        def.push_str(": ");
        def.push_str(&self.codify_type(krate, &constant.type_));
        def.push_str(" = ");
        // TODO: what about constant.value??
        def.push_str(&constant.expr);
        def.push(';');

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);
    }

    fn static_(&mut self, krate: &types::Crate, item: &Item, constant: &types::Static) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        self.append_header_for_inner_item("Static", item, summary);

        let meta = Self::codify_visibility(&item.visibility);
        let mut def = format!("{}static ", meta);
        if constant.mutable {
            def.push_str("mut ");
        }
        match &item.name {
            Some(name) => def.push_str(name),
            // FIXME: error handling.
            _ => panic!("Static without a name"),
        }
        def.push_str(": ");
        def.push_str(&self.codify_type(krate, &constant.type_));
        // TODO: or don't ignore `expr`?
        def.push(';');

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);
    }

    fn function(&mut self, krate: &types::Crate, item: &Item, function: &types::Function) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        let name = item.name
            .as_ref()
            .expect("Unnamed method");
        self.append_header_for_inner_item("Function", item, summary);

        let meta = Self::codify_visibility(&item.visibility);
        let header = self.codify_qualifiers(&function.header);
        let abi = Self::codify_abi(&function.abi);
        let signature = self.codify_fn_decl(krate, &function.decl);
        // FIXME: generics, bounds.
        let def = format!("{}{}{}fn {}{}", meta, header, abi, name, signature);

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);
    }

    fn enum_(&mut self, krate: &types::Crate, item: &Item, enum_: &types::Enum) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");

        // Avoid allocating too much below..
        if enum_.variants.len() >= 1_000_000 {
            panic!("Number of variants too large, considering opening a pull request to turn this into an iterative procedure.");
        }

        let meta = Self::codify_visibility(&item.visibility);
        let enum_name = self.name_for_item_at_path(&summary.path);

        let mut def: String = item.attrs
            .iter()
            .map(String::as_str)
            .interleave_shortest(std::iter::repeat("\n"))
            .collect();

        write!(&mut def, "{}enum {}", meta, enum_name)
            .expect("Writing to string succeeds");
        def.push_str(&self.codify_generics(krate, &enum_.generics));
        def.push_str(" {\n");
        self.append_header_for_inner_item("Enum", item, summary);

        let mut variant_documentation = vec![];
        for variant_id in &enum_.variants {
            if let Some(Item {
                inner: ItemEnum::VariantItem(variant),
                name: Some(name),
                visibility,
                docs,
                ..
            }) = krate.index.get(variant_id) {
                // Enum items do not have visibility for now..
                let _ = Self::codify_visibility(visibility);
                // FIXME: Different variant kinds.
                writeln!(&mut def, "    {},", name)
                    .expect("Writing to string succeeds");
                variant_documentation.push((name, variant, docs));
            } else {
                // FIXME: should not occur.
            }
        }

        if enum_.variants_stripped {
            def.push_str("    // some variants omitted\n");
        }
        def.push('}');

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);

        // FIXME: we would like a level-4 header..
        if !variant_documentation.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineStrong));
            self.buffered.push_back(Event::Text(Cow::Borrowed("Variants")));
            self.buffered.push_back(Event::End(Tag::InlineStrong));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        for (name, _variant, docs) in variant_documentation {
            self.buffered.push_back(Event::Start(Tag::Paragraph));

            self.buffered.push_back(Event::Start(Tag::InlineCode));
            // FIXME: struct variants, including links.
            self.buffered.push_back(Event::Text(Cow::Owned(name.clone())));
            self.buffered.push_back(Event::End(Tag::InlineCode));

            if let Some(docs) = docs {
                self.buffered.push_back(Event::Text(Cow::Borrowed("  ")));
                self.append_docs(docs.clone());
            }

            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        self.append_impls(krate, item, &enum_.impls);
    }

    fn trait_(&mut self, krate: &types::Crate, item: &Item, trait_: &types::Trait) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");

        // Avoid allocating too much below..
        if trait_.items.len() >= 1_000_000 {
            panic!("Number of fields too large, considering opening a pull request to turn this into an iterative procedure.");
        }

        let vis = Self::codify_visibility(&item.visibility);
        let safe = if trait_.is_unsafe { "unsafe " } else { "" };
        let auto = if trait_.is_auto { "auto " } else { "" };
        let trait_name = item.name.as_ref()
            .expect("Trait without name");

        let mut def: String = item.attrs
            .iter()
            .map(String::as_str)
            .interleave_shortest(std::iter::repeat("\n"))
            .collect();

        write!(&mut def, "{}{}{}trait {}", vis, safe, auto, trait_name)
            .expect("Writing to string succeeds");

        let generic_params = trait_.generics.params
            .iter()
            .map(|param| self.codify_generic_param(krate, param))
            .intersperse(String::from(", "));

        if !trait_.generics.params.is_empty() {
            def.push('<');
            def.push_str(&generic_params.collect::<String>());
            def.push('>');
        }

        let mut bounds = trait_.bounds
            .iter()
            .map(|bound| self.codify_generic_bound(krate, bound));

        if let Some(first) = bounds.next() {
            if trait_.bounds.len() > 2 {
                def.push_str("\n  : ");
                def.push_str(&first);
                let bounds = bounds
                    .intersperse(String::from("\n  + "))
                    .collect::<String>();
                def.push_str(&bounds);
            } else {
                def.push_str(": ");
                def.push_str(&first);
                for rest in bounds {
                    def.push_str(" + ");
                    def.push_str(&rest);
                }
            }
        }

        let predicates = trait_.generics.where_predicates
            .iter()
            .map(|pred| self.codify_generic_predicate(krate, pred))
            .intersperse(String::from(",\n    "));

        if !trait_.generics.where_predicates.is_empty() {
            def.push_str("\nwhere\n    ");
            def.push_str(&predicates.collect::<String>());
            def.push_str("\n");
        } else {
            def.push(' ');
        }

        self.append_header_for_inner_item("Trait", item, summary);

        // TODO: print replication of definition.
        let mut trait_items = vec![];
        // FIXME: bounds
        def.push_str("{\n");
        for item_id in &trait_.items {
            match krate.index.get(item_id) {
                Some(Item {
                    inner: ItemEnum::AssocTypeItem { bounds, default },
                    name: Some(name),
                    docs,
                    ..
                }) => {
                    def.push_str("    type ");
                    def.push_str(name);
                    let mut bounds = bounds.iter();
                    if let Some(first) = bounds.next() {
                        def.push_str(": ");
                        def.push_str(&self.codify_generic_bound(krate, first));
                        for rest in bounds {
                            def.push_str(" + ");
                            def.push_str(&self.codify_generic_bound(krate, rest));
                        }
                    }
                    if let Some(type_) = default {
                        def.push_str(" = ");
                        def.push_str(&self.codify_type(krate, type_));
                    }
                    def.push_str(";\n");

                    trait_items.push((name, docs));
                }
                Some(Item {
                    inner: ItemEnum::AssocConstItem { type_, default },
                    name: Some(name),
                    docs,
                    ..
                }) => {
                    def.push_str("    ");
                    def.push_str(name);
                    def.push_str(": ");
                    let type_ = self.codify_type(krate, type_);
                    def.push_str(&type_);
                    if let Some(default) = &default {
                        def.push_str(" = ");
                        def.push_str(default);
                    }
                    def.push_str(";\n");

                    trait_items.push((name, docs));
                }
                Some(Item {
                    inner: ItemEnum::MethodItem(method),
                    name: Some(name),
                    docs,
                    ..
                }) => {
                    def.push_str("    ");
                    def.push_str(&self.codify_qualifiers(&method.header));
                    def.push_str("fn ");
                    def.push_str(name);
                    // FIXME: generics
                    let type_ = self.codify_fn_decl(krate, &method.decl);
                    def.push_str(&type_);
                    // FIXME(rustdoc): show if it is defaulted?; as alternative for this terminator if so.
                    def.push_str(";\n");

                    trait_items.push((name, docs));
                }
                Some(other) => {
                    self.diagnostics
                        .warning(format!("Unhandled trait item: {:?}", other))
                        .note(format!("In {}", self.name_for_item_at_path(&summary.path)))
                        .emit();
                }
                None => unreachable!("Trait item does not exist?"),
            }
        }
        def.push('}');

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);

        // TODO: differentiate between constants, types, required methods, provided methods
        // FIXME: we would like a level-4 header..
        if !trait_items.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineStrong));
            self.buffered.push_back(Event::Text(Cow::Borrowed("Associated items")));
            self.buffered.push_back(Event::End(Tag::InlineStrong));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        // FIXME: add full declaration with links.
        for (name, docs) in trait_items {
            self.buffered.push_back(Event::Start(Tag::Paragraph));

            self.buffered.push_back(Event::Start(Tag::InlineCode));
            // FIXME: struct variants, including links.
            self.buffered.push_back(Event::Text(Cow::Owned(name.clone())));
            self.buffered.push_back(Event::End(Tag::InlineCode));

            if let Some(docs) = docs {
                self.buffered.push_back(Event::Text(Cow::Borrowed("  ")));
                self.append_docs(docs.clone())
            }

            self.buffered.push_back(Event::End(Tag::Paragraph));
        }
    }

    fn impl_(&mut self, krate: &types::Crate, item: &Item, impl_: &types::Impl) {
        let mut impl_header = String::from("impl");
        impl_header.push_str(&self.codify_generics(krate, &impl_.generics));
        // FIXME: generics
        impl_header.push(' ');
        if let Some(trait_) = &impl_.trait_ {
            if impl_.negative {
                impl_header.push('!');
            }
            impl_header.push_str(&self.codify_type(krate, trait_));
            impl_header.push_str(" for ");
        }
        impl_header.push_str(&self.codify_type(krate, &impl_.for_));

        self.buffered.push_back(Event::Start(Tag::Paragraph));
        self.buffered.push_back(Event::Start(Tag::InlineCode));
        self.buffered.push_back(Event::Text(Cow::Owned(impl_header)));
        self.buffered.push_back(Event::End(Tag::InlineCode));
        self.buffered.push_back(Event::End(Tag::Paragraph));

        if let Some(docs) = &item.docs {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Text(docs.clone().into()));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        let mut impl_items = vec![];

        for item_id in &impl_.items {
            match krate.index.get(item_id) {
                Some(Item {
                    inner: ItemEnum::TypedefItem(typedef),
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let mut def = format!("  {}type ", meta);
                    def.push_str(name);
                    def.push_str(" = ");
                    def.push_str(&self.codify_type(krate, &typedef.type_));
                    def.push_str(";\n");

                    impl_items.push((name, def, docs));
                }
                Some(Item {
                    inner: ItemEnum::ConstantItem(const_),
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let mut def = format!("  {}const ", meta);
                    def.push_str(name);
                    def.push_str(": ");
                    def.push_str(&self.codify_type(krate, &const_.type_));
                    def.push_str(" = ");
                    def.push_str(&const_.expr);
                    def.push_str(";\n");

                    impl_items.push((name, def, docs));
                }
                // FIXME(rustdoc): this is only due to an internal bug in rustdoc where associated
                // constants ( impl Type { pub const A: usize = 0 ) appear as AssocConstItem
                // instead which would be more appropriate for a trait.
                Some(Item {
                    inner: ItemEnum::AssocConstItem { type_, default: Some(const_def) },
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let mut def = format!("  {}const ", meta);
                    def.push_str(name);
                    def.push_str(": ");
                    def.push_str(&self.codify_type(krate, type_));
                    def.push_str(" = ");
                    def.push_str(const_def);
                    def.push_str(";\n");

                    impl_items.push((name, def, docs));
                }
                Some(Item {
                    inner: ItemEnum::MethodItem(method),
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let mut def = format!("  {}", meta);
                    def.push_str(&self.codify_qualifiers(&method.header));
                    def.push_str(" fn");
                    def.push_str(name);
                    def.push_str(&self.codify_generics(krate, &method.generics));
                    def.push_str(&self.codify_fn_decl(krate, &method.decl));

                    impl_items.push((name, def, docs));
                }
                Some(Item {
                    inner: ItemEnum::FunctionItem(function),
                    name: Some(name),
                    visibility,
                    docs,
                    ..
                }) => {
                    let meta = Self::codify_visibility(visibility);
                    let mut def = format!("  {}", meta);
                    def.push_str(&self.codify_qualifiers(&function.header));
                    def.push_str(" fn");
                    def.push_str(name);
                    def.push_str(&self.codify_generics(krate, &function.generics));
                    def.push_str(&self.codify_fn_decl(krate, &function.decl));

                    impl_items.push((name, def, docs));
                }
                Some(other) => {
                    self.diagnostics
                        .warning(format!("Unhandled impl item: {:?}", other))
                        .emit();
                }
                None => unreachable!("Trait item does not exist?"),
            }
        }

        for (_name, definition, docs) in impl_items {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineCode));
            self.buffered.push_back(Event::Text(Cow::Owned(definition)));
            self.buffered.push_back(Event::End(Tag::InlineCode));
            self.buffered.push_back(Event::End(Tag::Paragraph));

            if let Some(docs) = &docs {
                self.buffered.push_back(Event::Start(Tag::Paragraph));
                self.buffered.push_back(Event::Text(docs.clone().into()));
                self.buffered.push_back(Event::End(Tag::Paragraph));
            }
        }
    }

    fn typedef(&mut self, krate: &types::Crate, item: &Item, typedef: &types::Typedef) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");
        let name = item.name
            .as_ref()
            .expect("Typedef without name");

        self.append_header_for_inner_item("Typedef", item, summary);

        let meta = Self::codify_visibility(&item.visibility);
        let mut def = format!("{}type ", meta);
        def.push_str(name);
        def.push_str(" = ");
        def.push_str(&self.codify_type(krate, &typedef.type_));
        def.push(';');

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        self.append_item_docs(item);
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

    fn append_docs(&mut self, docs: String) {
        // Include recursively as Markdown.
        let include = ResolvedInclude::Markdown(
            self.simulated_source_path.clone(),
            self.context.clone(),
        );

        let parameter = Include {
            // Important: Scope sections under it.
            adjust_headers: true,
            // Rest of parameters are default.
            resolve_security: ResolveSecurity::Default,
            label: None,
            caption: None,
            title: None,
            alt_text: None,
            // FIXME: need a good way to express this?
            dst: "/".into(),
            scale: None,
            width: None,
            height: None,
        };
        let source = docs.into();
        self.buffered.push_back(Event::SyntheticInclude(include, parameter, source));
    }

    fn append_item_docs(&mut self, item: &types::Item) {
        if let Some(docs) = &item.docs {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.append_docs(docs.clone());
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }
    }

    fn codify_abi(abi: &str) -> String {
        match abi {
            "\"Rust\"" => String::new(),
            other => format!("extern {} ", other),
        }
    }

    fn codify_type(&self, krate: &types::Crate, type_: &types::Type) -> String {
        #[allow(clippy::enum_glob_use)]
        use types::Type::*;
        match type_ {
            ResolvedPath { name, args, param_names, .. } => {
                // FIXME: when name is empty, then the type should be named through its path!
                // That happens in things such as:
                // ```
                // impl<P> Deref for Type<P> {
                //     type Target = <P as Trait>::SomeType;
                // }
                // ```
                //
                // Here the Target is described as a QualifiedPath and the `trait_` attribute
                // refers to the `Trait` via a ResolvedPath without a name.
                let mut name = name.clone();
                match args.as_ref().map(|a| &**a) {
                    None => {},
                    Some(types::GenericArgs::AngleBracketed { args, bindings }) => {
                        let str_args = args
                            .iter()
                            .map(|arg| self.codify_generic_arg(krate, arg));

                        let str_bindings = bindings
                            .iter()
                            .map(|arg| self.codify_generic_binding(krate, arg));

                        let str_all = str_args
                            .chain(str_bindings)
                            .intersperse(String::from(", "));

                        if !args.is_empty() || !bindings.is_empty() {
                            name.push('<');
                            name.push_str(&str_all.collect::<String>());
                            name.push('>');
                        }
                    },
                    Some(types::GenericArgs::Parenthesized { .. }) => {
                        self.diagnostics
                            .warning("Encountered parenthesized type arguments, those are unimplemented")
                            .emit();
                    },
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
                let mut name = format!("({}", self.codify_type(krate, first));
                for type_ in items {
                    name.push(',');
                    name.push_str(&self.codify_type(krate, type_));
                }
                name.push(')');
                name
            },
            Slice(inner) => format!("[{}]", self.codify_type(krate, inner)),
            Array { type_, len } => {
                format!("[{}; {}]", self.codify_type(krate, type_), len)
            },
            // ImplTrait..
            Never => "!".into(),
            Infer => "_".into(),
            RawPointer { mutable, type_ } => {
                let qualifier = if *mutable { "mut" } else { "const" };
                format!("*{} {}", qualifier, self.codify_type(krate, type_))
            }
            BorrowedRef { lifetime, mutable, type_ } => {
                let lifetime = lifetime.as_ref().map_or_else(String::new, |st| format!("{} ", st));
                let qualifier = if *mutable { "mut " } else { "" };
                let type_ = self.codify_type(krate, type_);
                format!("&{}{}{}", lifetime, qualifier, type_)
            }
            QualifiedPath { name, self_type, trait_ } => {
                let self_type = self.codify_type(krate, self_type);
                let trait_ = self.codify_type(krate, trait_);
                format!("<{} as {}>::{}", self_type, trait_, name)
            }
            ImplTrait(bounds) => {
                let mut bounds = bounds.iter();
                if let Some(first) = bounds.next() {
                    let mut bound = String::from("impl ");
                    bound.push_str(&self.codify_generic_bound(krate, first));
                    for rest in bounds {
                        bound.push_str(" + ");
                        bound.push_str(&self.codify_generic_bound(krate, rest));
                    }
                    bound
                } else {
                    unreachable!("impl type without any named bounds");
                }
            }
            FunctionPointer(fnptr) => {
                // FIXME: for<'a> lifetime parameters.
                let mut base = self.codify_qualifiers(&fnptr.header);
                if !fnptr.abi.is_empty() {
                    base.push_str("extern ");
                    base.push_str(&fnptr.abi);
                    base.push(' ');
                }
                base.push_str(&self.codify_fn_decl(krate, &fnptr.decl));
                base
            }
        }
    }

    fn import(&mut self, krate: &types::Crate, item: &types::Item, import: &types::Import) {
        let summary = krate.paths.get(&item.id)
            // FIXME: this should fail and diagnose the rendering process, not panic.
            .expect("Bad item ID");

        self.append_header_for_inner_item("Import", item, summary);

        let mut def = Self::codify_visibility(&item.visibility);
        def.push_str("use ");
        def.push_str(&import.span);
        def.push_str(";");

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        if let Some(docs) = &item.docs {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Text(docs.clone().into()));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }
    }

    // FIXME(rustdoc) for some reason this also represents re-exports right now.
    //     pub mod inner { pub struct Outer; }
    //     pub use inner::Outer;
    fn extern_item(&mut self, krate: &types::Crate, item: &types::Item, import: &String, rename: Option<&String>) {
        let header = frontend::Header {
            label: WithRange(Cow::Borrowed(""), (0..0).into()),
            level: 2,
        };

        let meta = Self::codify_visibility(&item.visibility);
        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            Cow::Owned(format!("Re-export {}{}", meta, import))
        }));
        self.buffered.push_back(Event::End(Tag::Header(header.clone())));

        let mut def = Self::codify_visibility(&item.visibility);
        def.push_str("use ");
        def.push_str(&import);
        if let Some(rename) = rename {
            def.push_str(" as ");
            def.push_str(rename);
        }
        def.push_str(";");

        self.buffered.push_back(Event::Start(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));
        self.buffered.push_back(Event::Text(Cow::Owned(def)));
        self.buffered.push_back(Event::End(Tag::CodeBlock(Self::RUST_CODE_BLOCK)));

        if let Some(docs) = &item.docs {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Text(docs.clone().into()));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }
    }

    fn codify_fn_decl(&self, krate: &types::Crate, decl: &types::FnDecl) -> String {
        let inputs: Vec<_> = decl.inputs
            .iter()
            .map(|(name, type_)| {
                format!("{}: {}", name, self.codify_type(krate, type_))
            })
            .collect();

        let in_len: usize = inputs.iter().map(|st| st.chars().count()).sum();

        let output = if let Some(type_) = &decl.output {
            format!(" -> {}", self.codify_type(krate, type_))
        } else {
            "".into()
        };

        let out_len: usize = output.chars().count();

        // FIXME: this simplistic model counts unicode code points.
        // It would be more accurate to use something else.

        // We have three styles:
        // * (arg, arg, arg) -> Output
        // * (arg, arg, arg)
        //     -> Output
        // * (
        //       arg1,
        //       arg2,
        //   ) -> Output
        // *
        // Break around (), break between args, break before output.
        let (list_break, arg_break, out_break) = if in_len + out_len < 80 {
            // Same line for everything, arguments and output
            (false, false, false)
        } else if in_len < 120 {
            (true, false, true)
        } else {
            (true, true, false)
        };

        let mut decl = String::from("(");
        if list_break {
            decl.push_str("\n    ");
        }
        let mut inputs = inputs.into_iter();
        if let Some(first) = inputs.next() {
            decl.push_str(&first);
            for rest in inputs {
                if arg_break {
                    decl.push_str(",\n    ");
                } else {
                    decl.push_str(", ");
                }
                decl.push_str(&rest);
            }
        }
        if list_break {
            decl.push('\n');
        }
        decl.push(')');
        if out_break {
            decl.push_str("\n ");
        }
        decl.push_str(&output);
        decl
    }

    fn codify_generics(&self, krate: &types::Crate, bound: &types::Generics) -> String {
        let generics = bound.params
            .iter()
            .map(|param| self.codify_generic_param(krate, param))
            .intersperse(String::from(", "));

        let predicates = bound.where_predicates
            .iter()
            .map(|pred| self.codify_generic_predicate(krate, pred))
            .intersperse(String::from(",\n    "));

        match (bound.params.is_empty(), bound.where_predicates.is_empty()) {
            (false, false) => format!(
                "<{}>\nwhere\n    {}",
                generics.collect::<String>(),
                predicates.collect::<String>()
            ),
            (true, false) => format!("\nwhere\n    {}", predicates.collect::<String>()),
            (false, true) => format!("<{}>", generics.collect::<String>()),
            (true, true) => String::new(),
        }
    }

    fn codify_qualifiers(&self, quals: &HashSet<types::Qualifiers>) -> String {
        let mut quals = quals.clone();
        let mut stringified = String::new();

        if quals.remove(&types::Qualifiers::Async) {
            stringified.push_str(" async");
        }

        if quals.remove(&types::Qualifiers::Const) {
            stringified.push_str(" const");
        }

        if quals.remove(&types::Qualifiers::Unsafe) {
            stringified.push_str(" unsafe");
        }

        if let Some(non_space) = stringified.find(|c| c != ' ') {
            let _ = stringified.drain(..non_space);
        }

        stringified
    }

    fn codify_generic_param(&self, krate: &types::Crate, param: &types::GenericParamDef) -> String {
        match &param.kind {
            types::GenericParamDefKind::Lifetime => param.name.clone(),
            types::GenericParamDefKind::Type { bounds, default } => {
                let str_bounds = bounds
                    .iter()
                    .map(|bound| self.codify_generic_bound(krate, bound))
                    .intersperse(String::from(" + "));
                let mut def = param.name.clone();
                if !bounds.is_empty() {
                    def.push_str(": ");
                    def.push_str(&str_bounds.collect::<String>());
                }
                if let Some(default) = default {
                    def.push_str(" = ");
                    def.push_str(&self.codify_type(krate, default));
                }
                def
            },
            types::GenericParamDefKind::Const(const_) => {
                format!("const {}: {}", &param.name, self.codify_type(krate, const_))
            },
        }
    }

    fn codify_generic_predicate(&self, krate: &types::Crate, predicate: &types::WherePredicate) -> String {
        match predicate {
            types::WherePredicate::BoundPredicate { ty, bounds } => {
                let bound: String = bounds
                    .iter()
                    .map(|bound| self.codify_generic_bound(krate, bound))
                    .intersperse(String::from(" + "))
                    .collect();
                format!("{}: {}", self.codify_type(krate, ty), bound)
            },
            types::WherePredicate::RegionPredicate { lifetime, bounds } => {
                let bound: String = bounds
                    .iter()
                    .map(|bound| self.codify_generic_bound(krate, bound))
                    .intersperse(String::from(" + "))
                    .collect();
                format!("{}: {}", lifetime, bound)
            },
            types::WherePredicate::EqPredicate { lhs: _, rhs: _ } => {
                self.diagnostics
                    .warning("Equality bounds are not yet supported and will be ignored")
                    .emit();
                String::new()
            },
        }
    }

    fn codify_generic_arg(&self, krate: &types::Crate, arg: &types::GenericArg) -> String {
        match arg {
            types::GenericArg::Lifetime(lifetime) => lifetime.clone(),
            types::GenericArg::Type(type_) => self.codify_type(krate, type_),
            types::GenericArg::Const(constant) => {
                format!("{{{}}}", constant.expr)
            },
        }
    }

    fn codify_generic_binding(&self, krate: &types::Crate, binding: &types::TypeBinding) -> String {
        match &binding.binding {
            types::TypeBindingKind::Equality(rhs) => {
                format!("{}={}", &binding.name, &self.codify_type(krate, rhs))
            }
            types::TypeBindingKind::Constraint(bounds) => {
                let str_bounds = bounds
                    .iter()
                    .map(|bound| self.codify_generic_bound(krate, bound))
                    .intersperse(String::from(" + "));

                if bounds.is_empty() {
                    self.diagnostics
                        .warning("Generic constraint binding has no bounds?")
                        .emit();
                    String::new()
                } else {
                    format!("{}: {}", &binding.name, &str_bounds.collect::<String>())
                }
            }
        }
    }

    fn codify_generic_bound(&self, krate: &types::Crate, bound: &types::GenericBound) -> String {
        match bound {
            types::GenericBound::Outlives(lifetime) => lifetime.clone(),
            types::GenericBound::TraitBound { trait_, generic_params, modifier } => {
                if let types::TraitBoundModifier::None = modifier {} else {
                    // FIXME: implement this
                    self.diagnostics
                        .warning("Trait bound modifiers are not implemented")
                        .note(format!("Printing {:?}", modifier))
                        .emit();
                };

                if !generic_params.is_empty() {
                    // FIXME: implement this
                    self.diagnostics
                        .warning("Generic parameters are not implemented")
                        .note(format!("Omitting {} parameters for {:?}", generic_params.len(), trait_))
                        .emit();
                }

                self.codify_type(krate, trait_)
            }
        }
    }

    fn name_for_item_at_path(&self, path: &[String]) -> String {
        path.join("::")
    }

    fn label_for_item_at_path(&self, path: &[String]) -> String {
        path.join("-")
    }

    fn append_header_for_inner_item(
        &mut self,
        kind: &str,
        item: &Item,
        summary: &types::ItemSummary,
    ) {
        let label = self.label_for_item_at_path(&summary.path);

        let header = frontend::Header {
            label: WithRange(Cow::Owned(label.clone()), (0..0).into()),
            level: 2,
        };

        let meta = Self::codify_visibility(&item.visibility);
        self.buffered.push_back(Event::Start(Tag::Header(header.clone())));
        self.buffered.push_back(Event::Text({
            let const_name = self.name_for_item_at_path(&summary.path);
            Cow::Owned(format!("{} {}{}", kind,  meta, const_name))
        }));
        self.buffered.push_back(Event::End(Tag::Header(header.clone())));
    }

    fn append_impls(
        &mut self,
        krate: &types::Crate,
        item: &Item,
        impls: &[types::Id],
    ) {
        // FIXME: differentiate by kind with headers
        // let mut intrinsic = vec![];
        // let mut well_known = vec![];
        // let mut other_traits = vec![];

        if !impls.is_empty() {
            self.buffered.push_back(Event::Start(Tag::Paragraph));
            self.buffered.push_back(Event::Start(Tag::InlineStrong));
            self.buffered.push_back(Event::Text(Cow::Borrowed("Implementations")));
            self.buffered.push_back(Event::End(Tag::InlineStrong));
            self.buffered.push_back(Event::End(Tag::Paragraph));
        }

        for impl_ in impls.iter().rev() {
            self.stack.push(Traversal::Item(impl_.clone()));
        }
    }

    const RUST_CODE_BLOCK: frontend::CodeBlock<'static> = frontend::CodeBlock {
        label: None,
        caption: None,
        language: Some(WithRange(
            Cow::Borrowed("rust"),
            crate::frontend::range::SourceRange {
                start: 0,
                end: 0,
            }
        )),
    };
}
