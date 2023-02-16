use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{fold::Fold, parse_macro_input, parse_quote, ItemFn, Stmt};

#[proc_macro_attribute]
pub fn scratch_space(_: TokenStream, input: TokenStream) -> TokenStream {
    let fn_def = parse_macro_input!(input as ItemFn);
    let track_ident = Ident::new("scratch_local", Span::mixed_site());

    let mut fn_def = LocalInitFolder {
        track_ident: track_ident.clone(),
    }
    .fold_item_fn(fn_def);

    let init: Stmt = parse_quote! { let #track_ident: (); };

    fn_def.block.stmts.insert(0, init);

    quote! { #fn_def }.into()
}

struct LocalInitFolder {
    track_ident: Ident,
}

impl syn::fold::Fold for LocalInitFolder {
    fn fold_macro(&mut self, mut mac: syn::Macro) -> syn::Macro {
        if let Some(last_path) = mac.path.segments.iter().next_back() {
            match last_path.ident.to_string().as_str() {
                "scratch_write" => {
                    let track_ident = &self.track_ident.clone();
                    mac.path = parse_quote! { actual_scratch_write };
                    mac.tokens.extend(quote! { ; #track_ident });
                }
                "scratch_read" => {
                    let mut track_ident = self.track_ident.clone();
                    track_ident.set_span(track_ident.span().located_at(last_path.ident.span()));
                    mac.path = parse_quote! { actual_scratch_read };
                    mac.tokens.extend(quote! { ; #track_ident });
                }
                _ => {}
            }

            mac
        } else {
            mac
        }
    }
}
