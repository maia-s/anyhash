use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, ConstParam, Data, DeriveInput, Error, Fields, Generics, Index, Lifetime,
    LifetimeParam, Token, TypeParam, WhereClause,
};

fn crate_root() -> TokenStream {
    quote!(::hash_t)
}

#[proc_macro_derive(HashT)]
#[allow(non_snake_case)]
pub fn derive_hash_t(input: TokenStream1) -> TokenStream1 {
    let root = crate_root();
    let hash_t = quote!(#root::Hash);
    let hasher_t = quote!(#root::Hasher);
    let T = Ident::new("T", Span::mixed_site());

    let mut input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let mut tokens = TokenStream::new();
    let mut types = Vec::new();

    match input.data {
        Data::Struct(x) => match x.fields {
            Fields::Named(x) => {
                let fields = x.named.iter().map(|x| {
                    types.push(x.ty.clone());
                    x.ident.as_ref().unwrap()
                });
                quote! {
                    #( #hash_t::<#T>::hash(&self.#fields, state); )*
                }
                .to_tokens(&mut tokens)
            }

            Fields::Unnamed(x) => {
                let fields = x.unnamed.iter().enumerate().map(|(i, x)| {
                    types.push(x.ty.clone());
                    Index::from(i)
                });
                quote! {
                    #( #hash_t::<#T>::hash(&self.#fields, state); )*
                }
                .to_tokens(&mut tokens)
            }

            Fields::Unit => (),
        },

        Data::Enum(x) => {
            let mut variant_tokens = TokenStream::new();

            for x in x.variants.iter() {
                let var = &x.ident;

                match &x.fields {
                    Fields::Named(x) => {
                        let fields: Vec<_> = x
                            .named
                            .iter()
                            .map(|x| {
                                types.push(x.ty.clone());
                                x.ident.as_ref().unwrap()
                            })
                            .collect();
                        quote! {
                            Self::#var { #(#fields),* } => { #( #hash_t::<#T>::hash(#fields, state); )* }
                        }.to_tokens(&mut variant_tokens);
                    }

                    Fields::Unnamed(x) => {
                        let fields: Vec<_> = x
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, x)| {
                                types.push(x.ty.clone());
                                format_ident!("_{i}")
                            })
                            .collect();
                        quote! {
                            Self::#var(#(#fields),*) => { #( #hash_t::<#T>::hash(#fields, state); )* }
                        }.to_tokens(&mut variant_tokens);
                    }

                    Fields::Unit => quote! {
                        Self::#var => (),
                    }
                    .to_tokens(&mut variant_tokens),
                }
            }

            quote! {
                #hash_t::<#T>::hash(&core::mem::discriminant(self), state);
                match self {
                    #variant_tokens
                }
            }
            .to_tokens(&mut tokens);
        }

        Data::Union(_) => {
            return Error::new(ident.span(), "can't derive `Hash<T>` for union")
                .to_compile_error()
                .into()
        }
    }

    input.generics.make_where_clause();
    let wc = input.generics.where_clause.as_mut().unwrap();
    if !wc.predicates.empty_or_trailing() {
        wc.predicates.push_punct(<Token![,]>::default());
    }
    let where_ = where_(Some(wc));
    let SplitGenerics {
        lti,
        ltt,
        tpi,
        tpt,
        cpi,
        cpt,
        wc,
    } = split_generics(&input.generics);
    quote! {
        impl<#(#lti,)* #T #(,#tpi)* #(,#cpi)*> #hash_t<#T> for #ident<#(#ltt,)* #(#tpt,)* #(#cpt),*> #where_ #wc
            #( #types: #hash_t<#T> ),*
        {
            fn hash<H: #hasher_t<#T>>(&self, state: &mut H) {
                #tokens
            }
        }
    }
    .into()
}

fn where_(wc: Option<&WhereClause>) -> Option<Token![where]> {
    if let Some(wc) = wc {
        if wc.predicates.is_empty() {
            return Some(wc.where_token);
        }
    }
    None
}

struct SplitGenerics<
    'a,
    LTI: Iterator<Item = &'a LifetimeParam>,
    LTT: Iterator<Item = &'a Lifetime>,
    TPI: Iterator<Item = &'a TypeParam>,
    TPT: Iterator<Item = &'a Ident>,
    CPI: Iterator<Item = &'a ConstParam>,
    CPT: Iterator<Item = &'a Ident>,
> {
    lti: LTI,
    ltt: LTT,
    tpi: TPI,
    tpt: TPT,
    cpi: CPI,
    cpt: CPT,
    wc: &'a Option<WhereClause>,
}

fn split_generics(
    generics: &Generics,
) -> SplitGenerics<
    impl Iterator<Item = &LifetimeParam>,
    impl Iterator<Item = &Lifetime>,
    impl Iterator<Item = &TypeParam>,
    impl Iterator<Item = &Ident>,
    impl Iterator<Item = &ConstParam>,
    impl Iterator<Item = &Ident>,
> {
    SplitGenerics {
        lti: generics.lifetimes(),
        ltt: generics.lifetimes().map(|l| &l.lifetime),
        tpi: generics.type_params(),
        tpt: generics.type_params().map(|t| &t.ident),
        cpi: generics.const_params(),
        cpt: generics.const_params().map(|c| &c.ident),
        wc: &generics.where_clause,
    }
}
