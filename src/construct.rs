use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{attr_syntax::LetDefault, impl_syntax::AnonymousImpl};

pub(crate) fn generate(attr: &LetDefault, input: &AnonymousImpl) -> TokenStream {
    let pat_ident = &attr.pat_ident;
    let default = &attr.expr;
    let target = input.target();
    let ident = input.struct_name(attr);
    let state_ident = format_ident!("__anonymous_trait_state");
    let let_closures = input.methods().map(|method| {
        let ident = format_ident!("__anonymous_trait__{}", &method.sig.ident);
        let closure = crate::closure_expr::generate(target, method);
        quote! {
            #[allow(non_snake_case)]
            let mut #ident = #closure;
        }
    });
    let closures = input.methods().map(|method| {
        let method_ident = &method.sig.ident;
        let ident = format_ident!("__anonymous_trait__{}", &method.sig.ident);
        quote! {
            #method_ident: std::sync::Mutex::new(&mut #ident),
        }
    });
    quote! {
        let mut #state_ident = #default;
        #(#let_closures)*
        let #pat_ident = #ident {
            #state_ident: &mut #state_ident,
            #(#closures)*
        };
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
            let my_mock = Cat::default()
        };
        let input = parse_quote! {
            impl Something for Cat {}
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            let mut __anonymous_trait_state = Cat::default();
            let my_mock = my_mock__Something {
                __anonymous_trait_state: &mut __anonymous_trait_state,
            };
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn let_mut() {
        let attr = parse_quote! {
            let mut my_mock = Cat::default()
        };
        let input = parse_quote! {
            impl Something for Cat {}
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            let mut __anonymous_trait_state = Cat::default();
            let mut my_mock =my_mock__Something {
                __anonymous_trait_state: &mut __anonymous_trait_state,
            };
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn closures() {
        let attr = parse_quote! {
            let my_mock = Cat::default()
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&self) -> String {
                    "meow".to_string()
                }
                fn change_name(&mut self, name: String) {
                    self.name = name;
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            let mut __anonymous_trait_state = Cat::default();
            #[allow(non_snake_case)]
            let mut __anonymous_trait__meow = |__anonymous_trait_state: &Cat| -> String {
                "meow".to_string()
            };
            #[allow(non_snake_case)]
            let mut __anonymous_trait__change_name = |__anonymous_trait_state: &mut Cat, name: String| {
                __anonymous_trait_state.name = name;
            };
            let my_mock = my_mock__Something {
                __anonymous_trait_state: &mut __anonymous_trait_state,
                meow: std::sync::Mutex::new(&mut __anonymous_trait__meow),
                change_name: std::sync::Mutex::new(&mut __anonymous_trait__change_name),
            };
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn state_initial_value() {
        let attr = parse_quote! {
            let my_mock = Cat::new(aaa)
        };
        let input = parse_quote! {
            impl Something for Cat {
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            let mut __anonymous_trait_state = Cat::new(aaa);
            let my_mock = my_mock__Something {
                __anonymous_trait_state: &mut __anonymous_trait_state,
            };
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
