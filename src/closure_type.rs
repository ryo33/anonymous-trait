use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate(target: &syn::Type, input: &syn::ImplItemFn) -> TokenStream {
    let mut args = vec![];
    let bound = input
        .sig
        .asyncness
        .map_or_else(|| quote! {}, |_| quote! { + Send });
    if let Some(receiver) = input.sig.receiver() {
        let reference = receiver
            .reference
            .as_ref()
            .map(|(and_token, _lifetime)| and_token);
        let mutability = &receiver.mutability;
        args.push(quote! { #reference #mutability #target });
    }
    input.sig.inputs.iter().for_each(|arg| {
        let syn::FnArg::Typed(pat_type) = arg else {
            return;
        };
        let ty = if let syn::Type::Reference(reference) = pat_type.ty.as_ref() {
            // ignores lifetime
            let and_token = &reference.and_token;
            let elem = &reference.elem;
            quote! { #and_token #elem }
        } else {
            let ty = &pat_type.ty;
            quote! { #ty }
        };
        args.push(quote! { #ty });
    });
    let output = match &input.sig.output {
        syn::ReturnType::Default => quote! {},
        syn::ReturnType::Type(arrow, ty) => {
            let ty = if let syn::Type::Reference(reference) = ty.as_ref() {
                // ignores lifetime and use 'static instead
                let and_token = &reference.and_token;
                let elem = &reference.elem;
                quote! { #and_token 'static #elem }
            } else {
                let ty = &ty;
                quote! { #ty }
            };
            quote! { #arrow #ty }
        }
    };
    quote! {
        FnMut(#(#args),*) #output #bound
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn empty() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow() {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut()
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn self_by_ref() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(&self) {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(&Cat)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn self_by_mut_ref() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(&mut self) {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(&mut Cat)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn self_by_value() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(self) {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(Cat)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(name: String, count: usize) {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(String, usize)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments_with_ref() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(name: &str, count: usize) {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(&str, usize)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments_with_self() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(&self, name: String, count: usize) {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(&Cat, String, usize)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments_with_lifetime() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow<'a>(&self, name: &'a str, count: usize) {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut(&Cat, &str, usize)
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn output() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow() -> String {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut() -> String
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn output_must_be_static() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow() -> &str {
                "meow"
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            FnMut() -> &'static str
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
