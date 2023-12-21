use quote::ToTokens as _;
use syn::{parse::Parse, Token};

#[allow(dead_code)]
pub(crate) struct LetDefault {
    pub let_token: Token![let],
    pub pat_ident: syn::PatIdent,
    pub eq_token: Token![=],
    pub expr: syn::Expr,
}

impl Parse for LetDefault {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: syn::ExprLet = input.parse()?;
        let syn::Pat::Ident(pat_ident) = *expr.pat else {
            let span = expr
                .pat
                .to_token_stream()
                .into_iter()
                .next()
                .expect("not empty")
                .span();
            return Err(syn::Error::new(span, "expected identifier"));
        };
        Ok(Self {
            let_token: expr.let_token,
            pat_ident,
            eq_token: expr.eq_token,
            expr: *expr.expr,
        })
    }
}
