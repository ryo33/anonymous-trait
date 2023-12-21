use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};

pub(crate) fn generate(target: &syn::Type, input: &syn::ImplItemFn) -> TokenStream {
    let mut args = vec![];
    if let Some(receiver) = input.sig.receiver() {
        // ignores lifetime
        let reference = receiver
            .reference
            .as_ref()
            .map(|(and_token, _lifetime)| and_token);
        let mutability = &receiver.mutability;
        args.push(quote! { __anonymous_trait_state: #reference #mutability #target })
    };
    input.sig.inputs.iter().for_each(|arg| {
        let syn::FnArg::Typed(pat_type) = arg else {
            return;
        };
        let pat = &pat_type.pat;
        let ty = if let syn::Type::Reference(reference) = pat_type.ty.as_ref() {
            // ignores lifetime
            let and_token = &reference.and_token;
            let elem = &reference.elem;
            quote! { #and_token #elem }
        } else {
            let ty = &pat_type.ty;
            quote! { #ty }
        };
        args.push(quote! { #pat: #ty })
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
    let block = input
        .block
        .to_token_stream()
        .into_iter()
        .map(replace_self)
        .collect::<TokenStream>();
    quote! {
        |#(#args),*| #output #block
    }
}

fn replace_self(token: TokenTree) -> TokenTree {
    match token {
        TokenTree::Ident(ident) => {
            if ident == "self" {
                TokenTree::Ident(syn::Ident::new("__anonymous_trait_state", Span::call_site()))
            } else {
                TokenTree::Ident(ident)
            }
        }
        TokenTree::Group(group) => TokenTree::Group(Group::new(
            group.delimiter(),
            group.stream().into_iter().map(replace_self).collect(),
        )),
        TokenTree::Punct(punct) => TokenTree::Punct(punct),
        TokenTree::Literal(literal) => TokenTree::Literal(literal),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn meow(&self) -> String {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat| -> String {
                "meow".to_string()
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn mut_self() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn change_name(&mut self) {
                self.name = name;
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &mut Cat| {
                __anonymous_trait_state.name = name;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn no_self() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow() -> String {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            | | -> String {
                "meow".to_string()
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(name: String, count: usize) -> String {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |name: String, count: usize| -> String {
                format!("{}: meow {}", name, count)
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn arguments_with_self() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(&self, name: String, count: usize) -> String {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat, name: String, count: usize| -> String {
                format!("{}: meow {}", name, count)
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn replace_self_to_narrative_state() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(&self) -> String {
                call(self.name);
                call(&self.name);
                self.name();
                call(self);
                format!("{}", self);
                format!("{}", (self, self));
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat| -> String {
                call(__anonymous_trait_state.name);
                call(&__anonymous_trait_state.name);
                __anonymous_trait_state.name();
                call(__anonymous_trait_state);
                format!("{}", __anonymous_trait_state);
                format!("{}", (__anonymous_trait_state, __anonymous_trait_state));
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn no_return() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(&self) {
                call(self.name);
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat| {
                call(__anonymous_trait_state.name);
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn remove_lifetimes_from_arguments() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow<'a>(&self, name: &'a String, count: usize) {
                format!("{}: meow {}", name, count)
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat, name: &String, count: usize| {
                format!("{}: meow {}", name, count)
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn make_static_output_reference() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(&self) -> &str {
                "meow"
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat| -> &'static str {
                "meow"
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn owned_self() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow(self) -> String {
                "meow".to_string()
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: Cat| -> String {
                "meow".to_string()
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn self_with_lifetime() {
        let target = parse_quote!(Cat);
        let input = parse_quote! {
            fn call_meow<'a>(&'a self) -> &'a str {
                "meow"
            }
        };
        let actual = generate(&target, &input);
        let expected = quote! {
            |__anonymous_trait_state: &Cat| -> &'static str {
                "meow"
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
