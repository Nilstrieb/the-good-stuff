use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, ForeignItem, ForeignItemFn, ItemFn, Pat, PatIdent, PatType, Visibility,
};

pub fn safe_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut foreign = parse_macro_input!(input as syn::ItemForeignMod);

    let mut safe_wrappers = Vec::new();
    let src_items = std::mem::take(&mut foreign.items);

    for item in src_items {
        match item {
            ForeignItem::Fn(item_fn) => {
                let (replacement, safe_wrapper) = mangle_ident_and_add_link_name(item_fn);
                foreign.items.push(ForeignItem::Fn(replacement));

                safe_wrappers.push(safe_wrapper);
            }
            item => match head_span_foreign_item(&item) {
                Some(span) => {
                    return quote_spanned! {
                        span => compile_error! { "only foreign functions are allowed" }
                    }
                    .into();
                }
                None => {
                    return quote! {
                        compile_error! { "only foreign functions are allowed" }
                    }
                    .into();
                }
            },
        }
    }

    quote! { #foreign #(#safe_wrappers)* }.into()
}

fn mangle_ident_and_add_link_name(mut item: ForeignItemFn) -> (ForeignItemFn, ItemFn) {
    if item.attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map_or(false, |ident| ident.to_string() == "link_name")
    }) {
        panic!("oh no you have alink name already")
    }

    let vis = std::mem::replace(&mut item.vis, Visibility::Inherited);

    let name = item.sig.ident;
    let name_str = name.to_string();
    if name_str.starts_with("r#") {
        panic!("rawr :>(");
    }

    let mangled = format!("_safe_extern_inner_{name_str}");
    let new_name = Ident::new(&mangled, name.span());
    item.sig.ident = new_name.clone();

    item.attrs
        .push(syn::parse_quote! { #[link_name = #name_str] });

    let args = item.sig.inputs.iter().map(|param| match param {
        syn::FnArg::Receiver(_) => panic!("cannot have reciver in foreign function"),
        syn::FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(PatIdent { ident, .. }) => quote! { #ident },
            _ => panic!("invalid argument in foreign function"),
        },
    });

    let mut safe_sig = item.sig.clone();
    safe_sig.ident = name;
    let safe_wrapper = ItemFn {
        attrs: Vec::new(),
        vis,
        sig: safe_sig,
        block: syn::parse_quote! {
            {
                unsafe { #new_name(#(#args),*) }
            }
        },
    };

    (item, safe_wrapper)
}

fn head_span_foreign_item(item: &ForeignItem) -> Option<proc_macro2::Span> {
    Some(match item {
        ForeignItem::Fn(_) => unreachable!(),
        ForeignItem::Static(s) => s.static_token.span,
        ForeignItem::Type(ty) => ty.type_token.span,
        ForeignItem::Macro(m) => m.mac.path.segments[0].ident.span(),
        _ => return None,
    })
}
