use proc_macro2::TokenStream;
use quote::quote;

use crate::{attr_syntax::LetDefault, impl_syntax::AnonymousImpl};

pub(crate) fn generate(attr: &LetDefault, mock: &AnonymousImpl) -> TokenStream {
    let trait_ = &mock.trait_;
    let struct_name = mock.struct_name(attr);
    let lifetime = quote!('__anonymous_trait_state);
    let generics = mock.methods().map(|method| {
        let method_ident = &method.sig.ident;
        let closure_type = crate::closure_type::generate(mock.target(), method);
        quote! {
            #method_ident: #closure_type,
        }
    });
    let struct_generics = mock.methods().map(|method| {
        let method_ident = &method.sig.ident;
        quote! {
            #method_ident
        }
    });
    let methods = mock.methods().map(|method| {
        let method_ident = &method.sig.ident;
        let receiver = method.sig.receiver();
        let args = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                let syn::FnArg::Typed(pat_type) = arg else {
                    return None;
                };
                Some(pat_type)
            })
            .collect::<Vec<_>>();
        let arg_pats = args.iter().map(|arg| &arg.pat);
        let output = &method.sig.output;
        quote! {
            fn #method_ident(#receiver #(,#args)*) #output {
                self.#method_ident.lock().unwrap()(self.__anonymous_trait_state #(,#arg_pats)*)
            }
        }
    });
    quote! {
        #[allow(non_camel_case_types)]
        impl <
            #lifetime,
            #(#generics)*
        > #trait_ for #struct_name<#lifetime #(,#struct_generics)*> {
            #(#methods)*
        }
    }
}

#[cfg(test)]
mod tests {
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
            impl <
                '__anonymous_trait_state,
            > Something for my_mock__Something<'__anonymous_trait_state> {
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn with_method() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&self) -> String {
                    "meow".to_string()
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            impl <
                '__anonymous_trait_state,
                meow: FnMut(&Cat) -> String,
            > Something for my_mock__Something<'__anonymous_trait_state, meow> {
                fn meow(&self) -> String {
                    self.meow.lock().unwrap()(self.__anonymous_trait_state)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn method_args() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&self, volume: u8, count: usize) -> String {
                    "meow".to_string()
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            impl <
                '__anonymous_trait_state,
                meow: FnMut(&Cat, u8, usize) -> String,
            > Something for my_mock__Something<'__anonymous_trait_state, meow> {
                fn meow(&self, volume: u8, count: usize) -> String {
                    self.meow.lock().unwrap()(self.__anonymous_trait_state, volume, count)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn receiver_mut() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&mut self) -> String {
                    "meow".to_string()
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            impl <
                '__anonymous_trait_state,
                meow: FnMut(&mut Cat) -> String,
            > Something for my_mock__Something<'__anonymous_trait_state, meow> {
                fn meow(&mut self) -> String {
                    self.meow.lock().unwrap()(self.__anonymous_trait_state)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn no_output() {
        let attr = parse_quote! {
            let my_mock = Cat
        };
        let input = parse_quote! {
            impl Something for Cat {
                fn meow(&self) {
                    "meow".to_string()
                }
            }
        };
        let actual = generate(&attr, &input);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            impl <
                '__anonymous_trait_state,
                meow: FnMut(&Cat),
            > Something for my_mock__Something<'__anonymous_trait_state, meow> {
                fn meow(&self) {
                    self.meow.lock().unwrap()(self.__anonymous_trait_state)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string())
    }
}
