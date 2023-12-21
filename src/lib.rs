mod attr_syntax;
mod closure_expr;
mod closure_type;
mod construct;
mod impl_syntax;
mod impl_trait;
mod packed_struct;

use attr_syntax::LetDefault;
use impl_syntax::AnonymousImpl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn anonymous_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as LetDefault);
    let input = syn::parse_macro_input!(item as AnonymousImpl);
    let packed_struct = packed_struct::generate(&attr, &input);
    let impl_trait = impl_trait::generate(&attr, &input);
    let construct = construct::generate(&attr, &input);
    quote::quote! {
        #packed_struct
        #impl_trait
        #construct
    }
    .into()
}
