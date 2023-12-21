use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{attr_syntax::LetDefault, impl_syntax::AnonymousImpl};

pub(crate) fn generate(attr: &LetDefault, input: &AnonymousImpl) -> TokenStream {
    let target = input.target();
    let ident = input.struct_name(attr);
    let state_ident = format_ident!("__anonymous_trait_state");
    let state_lifetime = quote!('__anonymous_trait_state);
    let generics = input.methods().map(|method| {
        let method_ident = &method.sig.ident;
        let closure_type = crate::closure_type::generate(target, method);
        quote! {
            #method_ident: #closure_type,
        }
    });
    let fields = input.methods().map(|method| {
        let method_ident = &method.sig.ident;
        quote! {
            #method_ident: std::sync::Mutex<&#state_lifetime mut #method_ident>,
        }
    });
    quote! {
        #[allow(non_camel_case_types)]
        struct #ident<
            #state_lifetime,
            #(#generics)*
        > {
            #state_ident: & #state_lifetime mut #target,
            #(#fields)*
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn empty() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {}
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            struct my_mock__Something<
                '__anonymous_trait_state,
            > {
                __anonymous_trait_state: &'__anonymous_trait_state mut Cat,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn fields() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&self) -> String {
                    unimplemented!()
                }
                fn change_name(&mut self, name: String) {
                    unimplemented!()
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            struct my_mock__Something<
                '__anonymous_trait_state,
                meow: FnMut(&Cat) -> String,
                change_name: FnMut(&mut Cat, String),
            > {
                __anonymous_trait_state: &'__anonymous_trait_state mut Cat,
                meow: std::sync::Mutex<&'__anonymous_trait_state mut meow>,
                change_name: std::sync::Mutex<&'__anonymous_trait_state mut change_name>,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
