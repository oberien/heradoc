use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct Crate {
    name: String,
    version: String,
    includes_private: bool,
    root: Id,
    index: HashMap<Id, Item>,
    paths: HashMap<Id, ItemSummary>,
    extern_crates: HashMap<Id, ExternalCrate>,
    format_version: u32,
}

#[derive(Deserialize)]
struct Item {
    crate_id: u32,
    name: String,
    span: Option<Span>,
    visibility: String,
    docs: String,
    links: HashMap<String, Id>,
    attrs: Vec<String>,
    deprecation: Option<Deprecation>,
    #[serde(flatten)]
    inner: ItemInner ,
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "inner")]
enum ItemInner {
    #[serde(rename = "module")]
    Module {
        items: Vec<Id>,
    },
    #[serde(rename = "function")]
    Function {
        decl: FnDecl,
        generics: Generics,
        header: String,
        abi: String,
    },
    #[serde(rename = "struct")]
    Struct {
        struct_type: String,
        generics: Generics,
        fields_stripped: bool,
        fields: Vec<Id>,
        impls: Vec<Id>,
    },
    #[serde(rename = "union")]
    Union {
        struct_type: String,
        generics: Generics,
        fields_stripped: bool,
        fields: Vec<Id>,
        impls: Vec<Id>,
    },
    #[serde(rename = "struct_field")]
    StructField {
        r#type: Type,
    },
    #[serde(rename = "enum")]
    Enum {
        generics: Generics,
        fields: Vec<Id>,
        fields_stripped: bool,
        impls: Vec<Id>,
    },
    #[serde(rename = "variant")]
    Variant {
        // FIXME:
    },
    #[serde(rename = "trait")]
    Trait {
        is_auto: bool,
        is_unsafe: bool,
        items: Vec<Id>,
        generics: Generics,
        bounds: Vec<GenericBound>,
    },
    #[serde(rename = "trait_alias")]
    TraitAlias {
        generics: Generics,
        bounds: Vec<GenericBound>,
    },
    #[serde(rename = "method")]
    Method {
        decl: FnDecl,
        generics: Generics,
        header: String,
        has_body: bool,
    },
    #[serde(rename = "assoc_const")]
    AssocConst {
        r#type: Type,
        default: Option<String>,
    },
    #[serde(rename = "assoc_type")]
    AssocType {
        bounds: Vec<GenericBound>,
        default: Option<Type>,
    },
    #[serde(rename = "impl")]
    Impl {
        is_unsafe: bool,
        generics: Generics,
        provided_trait_methods: Vec<String>,
        r#trait: Option<Type>,
        r#for: Type,
        items: Vec<Id>,
        negative: bool,
        synthetic: bool,
        blanket_impl: Option<String>,
    },
    #[serde(rename = "constant")]
    Constant {
        r#type: Type,
        expr: String,
        value: Option<String>,
        is_literal: bool,
    },
    #[serde(rename = "static")]
    Static {
        r#type: Type,
        expr: String,
        mutable: bool,
    },
    #[serde(rename = "typedef")]
    Typedef {
        r#type: Type,
        generics: Generics,
    },
    #[serde(rename = "opaque_ty")]
    OpaqueTy {
        bounds: Vec<GenericBound>,
        generics: Generics,
    },
    #[serde(rename = "foreign_type")]
    ForeignType(),
    #[serde(rename = "extern_crate")]
    ExternCrate {
        name: String,
        rename: Option<String>,
    },
    #[serde(rename = "import")]
    Import {
        source: String,
        name: String,
        id: Id,
        glob: bool,
    },
    #[serde(rename = "macro")]
    Macro(String),
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "inner")]
enum Type {
    // FIXME: resoled_path
    #[serde(rename = "resolved_path")]
    ResolvedPath {
        name: String,
        args: Option<Box<GenericArgs>>,
        id: Id,
        param_names: Box<GenericBound>,
    },
    #[serde(rename = "generic")]
    Generic(String),
    #[serde(rename = "tuple")]
    Tuple(Vec<Type>),
    #[serde(rename = "slice")]
    Slice(Box<Type>),
    #[serde(rename = "array")]
    Array {
        r#type: Box<Type>,
        len: String,
    },
    #[serde(rename = "impl_trait")]
    ImplTrait(Vec<GenericBound>),
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "infer")]
    Infer,
    #[serde(rename = "function_pointer")]
    FunctionPointer {
        is_unsafe: bool,
        decl: Box<FnDecl>,
        params: Vec<GenericParamDef>,
        abi: String,
    },
    #[serde(rename = "raw_pointer")]
    RawPointer {
        mutable: bool,
        r#type: Box<Type>,
    },
    #[serde(rename = "borrowed_ref")]
    BorrowedRef {
        lifetime: Option<String>,
        mutable: bool,
        r#type: Box<Type>,
    },
    #[serde(rename = "qualified_path")]
    QualifiedPath {
        name: String,
        self_type: Box<Type>,
        r#trait: Box<Type>,
    },
}

#[derive(Deserialize)]
enum GenericArgs {
    #[serde(rename = "angle_bracketed")]
    AngleBracketed {
        args: Vec<GenericArg>,
        bindings: TypeBinding,
    },
    #[serde(rename = "paranthesized")]
    Paranthesized {
        inputs: Vec<Type>,
        output: Type,
    },
}

#[derive(Deserialize)]
struct TypeBinding {
    name: String,
    binding: TypeBindingInner,
}

#[derive(Deserialize)]
enum TypeBindingInner {
    #[serde(rename = "equality")]
    Equality(Type),
    #[serde(rename = "constraint")]
    Constraint(Vec<GenericBound>),
}

#[derive(Deserialize)]
enum GenericArg {
    #[serde(rename = "lifetime")]
    Lifetime(String),
    #[serde(rename = "type")]
    Type(Type),
    #[serde(rename = "const")]
    Const {
        r#type: Type,
        expr: String,
        value: Option<String>,
        is_literal: bool,
    },
}

#[derive(Deserialize)]
struct ItemSummary {
    crate_id: u32,
    path: Vec<String>,
    kind: String,
}

#[derive(Deserialize)]
struct ExternalCrate {
    name: String,
    html_root_url: String,
}

#[derive(Deserialize)]
struct Visibility {
    parent: Id,
    path: String,
}

#[derive(Deserialize)]
struct Span {
    filename: String,
    begin: (u32, u32),
    end: (u32, u32),
}

#[derive(Deserialize)]
struct Deprecation {
    since: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct FnDecl {
    inputs: Vec<(String, Type)>,
    output: Option<Type>,
    c_variadic: bool,
}

#[derive(Deserialize)]
struct Generics {
    params: Vec<GenericParamDef>,
    where_predicates: Vec<WherePredicate>,
}

#[derive(Deserialize)]
struct GenericParamDef {
    name: String,
    kind: GenericParamDefInner,
}

#[derive(Deserialize)]
enum GenericParamDefInner {
    #[serde(rename = "lifetime")]
    Lifetime,
    #[serde(rename = "const")]
    Const(Type),
    #[serde(rename = "type")]
    Type {
        bounds: Vec<GenericBound>,
        default: Option<Type>,
        // TODO: this appears in RFC but not in the spec.
        synthetic: Option<bool>,
    },
}

#[derive(Deserialize)]
enum WherePredicate {
    #[serde(rename = "bound_predicate")]
    Bound {
        ty: Type,
        bounds: Vec<GenericBound>,
    },
    #[serde(rename = "region_predicate")]
    Region {
        lifetime: String,
        bounds: Vec<GenericBound>,
    },
    #[serde(rename = "eq_predicate")]
    Eq {
        lhs: Type,
        rhs: Type,
    },
}

#[derive(Deserialize)]
enum GenericBound {
    #[serde(rename = "trait_bound")]
    TraitBound {
        r#trait: Type,
        modifier: String,
        generics_params: Vec<GenericParamDef>,
    }
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
struct Id(String);
