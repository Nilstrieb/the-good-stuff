use proc_macro::TokenStream;

mod safe_extern;
mod scratch;

#[proc_macro_attribute]
pub fn scratch_space(attr: TokenStream, input: TokenStream) -> TokenStream {
    scratch::scratch_space(attr, input)
}

/// # safe-extern
///
/// Mark foreign functions as to be safe to call.
///
/// ```ignore
/// #[safe_extern]
/// extern "Rust" {
///     fn add(a: u8, b: u8) -> u8;
/// }
///
/// fn main() {
///     assert_eq!(add(1, 2), 3);
/// }
/// ```
///
/// It works by expanding the above to this
///
/// ```ignore
/// extern "Rust" {
///     #[link_name = "add"]
///     fn _safe_extern_inner_add(a: u8, b: u8) -> u8;
/// }
/// fn add(a: u8, b: u8) -> u8 {
///     unsafe { _safe_extern_inner_add(a, b) }
/// }
///
/// fn main() {
///     assert_eq!(add(1, 2), 3);
/// }
/// ```
///
/// This is of course unsound and the macro needs to be `unsafe` somehow but I can't be bothered with that right now lol.
#[proc_macro_attribute]
pub fn safe_extern(attr: TokenStream, input: TokenStream) -> TokenStream {
    safe_extern::safe_extern(attr, input)
}
