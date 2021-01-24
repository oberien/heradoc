#![allow(dead_code)]
#![allow(unused_variables)]

/// Documented module abc.
pub mod abc {
    /// A simple type.
    pub struct SimpleType;
}

/// A struct with repr annotation.
#[repr(C)]
pub struct KindOfReprC(pub u8);

/// This is used as a return type.
pub struct ReturnType;

/// An enum with different variants.
pub enum AnEnum {
    VariantA,
    VariantB,
    VariantStruct {
        a: ReturnType,
        b: usize,
    },
}

/// Really, a simple union, basically MaybeUninit.
pub union TestUnion {
    /// Use this to make everything be uninitialized.
    pub a: (),
    pub b: usize,
}

/// An amazing trait item with a few features.
pub trait ThisIsATrait: Clone {
    /// An associated constant.
    const CONSTANT: usize;
    /// An associated type.
    type Type: Copy;
    /// An ordinary method taking `&self`.
    fn method(&self, param: usize);
    /// A generic method with explicit bound.
    fn generic<T>(&self, t: T) -> usize
        where T: Copy;
    /// An ordinary static method.
    fn static_method(a: usize);
    /// A method that should have an unsafe qualifier.
    unsafe fn unsafe_method();
    /// A method that should have extern "C".
    extern "C" fn extern_method();
    /// A method provided by default by the trait.
    fn provided(&self) -> usize {
        0
    }
}

/// A normal function.
pub fn function_item(param: usize) -> ReturnType {
    ReturnType
}

/// A function with a very long signature.
pub fn complicated_function(
    long_parameter_name_but_still_okay: usize,
    oh_no_a_second_parameter: usize,
    and_a_third_one_definitely_makes_the_signature_long: usize,
) -> usize {
    0
}

/// A function with a non-Rust ABI.
pub extern "C" fn extern_c() {}

/// A static, not a constant.
pub static A_STATIC: KindOfReprC = KindOfReprC(0);

/// A constant with some arbitrary type.
pub const A_CONST: KindOfReprC = KindOfReprC(0u8);
/// A literal number.
pub const A_CONST_LITERAL: u8 = 0u8;
/// Wow, more literals.
pub const A_STR_LITERAL: &'static str = "string literal";
/// So hidden from public view.
const PRIVATE_CONST: u8 = 0;
