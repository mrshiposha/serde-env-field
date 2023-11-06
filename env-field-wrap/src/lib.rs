use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput,
    GenericArgument, PathArguments, PathSegment, Token,
};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn env_field_wrap(params: TokenStream, input: TokenStream) -> TokenStream {
    if !params.is_empty() {
        abort_call_site!("The `env_field_wrap` doesn't take any parameters");
    }

    let input = parse_macro_input!(input as DeriveInput);

    let attrs = attrs_tokens(input.attrs);

    let vis = input.vis;
    let ident = input.ident;
    let generics = input.generics;

    let (item_tok, data_with_env_fields) = match input.data {
        Data::Struct(data) => (quote![struct], wrap_fields(data.fields, WrapKind::Struct)),
        Data::Enum(data) => (quote![enum], enum_env_field_wrap(data)),
        Data::Union(data) => abort!(data.union_token, "unions are not supported"),
    };

    quote! {
        #attrs
        #vis
        #item_tok
        #ident
        #generics
        #data_with_env_fields
    }
    .into()
}

fn attrs_tokens(attrs: Vec<syn::Attribute>) -> TokenStream2 {
    let mut attrs_tokens = TokenStream2::new();
    for attr in attrs {
        attr.to_tokens(&mut attrs_tokens);
    }

    attrs_tokens
}

enum WrapAttr {
    Skip,
    GenericsOnly(Span),
}

fn take_env_field_wrap_attr(attrs: &mut Vec<syn::Attribute>) -> Option<WrapAttr> {
    let mut index = 0;
    let wrap_attr = attrs.iter().find_map(|attr| match &attr.meta {
        syn::Meta::List(list) => list.path.get_ident().and_then(|ident| {
            (ident == "env_field_wrap").then_some((list.span(), list.tokens.to_string()))
        }),
        _ => {
            index += 1;
            None
        }
    });

    if wrap_attr.is_some() {
        attrs.remove(index);
    }

    wrap_attr.and_then(|(span, wrap_attr)| match wrap_attr.as_str() {
        "skip" => Some(WrapAttr::Skip),
        "generics_only" => Some(WrapAttr::GenericsOnly(span)),
        _ => None,
    })
}

fn is_type(ty: &syn::Type, ty_paths: &[&str]) -> bool {
    match ty {
        syn::Type::Path(ty_path) if ty_path.qself.is_none() => {
            let path = &ty_path.path;

            let path_ty_str = path.segments.iter().fold(String::new(), |mut acc, seg| {
                acc.push_str(&seg.ident.to_string());
                acc.push_str("::");
                acc
            });

            // Remove the last `::`
            let path_ty_str = &path_ty_str[..path_ty_str.len() - 2];

            ty_paths.iter().any(|ty| *ty == path_ty_str)
        }
        _ => false,
    }
}

fn wrap_generics_only(ty: &syn::Type) -> TokenStream2 {
    match ty {
        syn::Type::Path(ty) => {
            if let Some(qself) = &ty.qself {
                abort!(
                    qself.span(),
                    "generics_only: a plan type path with generics is expected"
                );
            }

            let path = &ty.path;

            let segments = path.segments.iter();
            let mut leading_segments = Punctuated::<PathSegment, Token![::]>::new();
            let mut ty_with_generics = None;

            for segment in segments {
                match &segment.arguments {
                    PathArguments::None => leading_segments.push(segment.clone()),
                    PathArguments::AngleBracketed(angle_args) => {
                        let wrapped_generics = angle_args
                            .args
                            .iter()
                            .map(|arg| match arg {
                                GenericArgument::Type(generic) => {
                                    quote!(::serde_env_field::EnvField<#generic>)
                                }
                                _ => abort!(angle_args.args, "generics_only: a type is expected"),
                            })
                            .collect::<Punctuated<_, Token![,]>>();

                        let ident = &segment.ident;
                        ty_with_generics = Some(quote!(#ident < #wrapped_generics >));
                    }
                    _ => abort!(
                        segment.arguments,
                        "generics_only: unexpected type arguments"
                    ),
                }
            }

            if ty_with_generics.is_none() {
                abort!(ty, "generics_only: no generics found");
            }

            leading_segments.pop_punct();

            let leading_colon = path.leading_colon;
            let ty_path = if leading_segments.is_empty() {
                quote!(#ty_with_generics)
            } else {
                quote!(#leading_segments :: #ty_with_generics)
            };

            quote! {
                #leading_colon #ty_path
            }
        }
        _ => abort!(ty, "generics_only: a type with generic(s) is expected"),
    }
}

fn process_fields(fields: impl Iterator<Item = syn::Field>) -> TokenStream2 {
    fields
        .map(|mut field| {
            let wrap_attr = take_env_field_wrap_attr(&mut field.attrs);

            let ty: syn::Type = field.ty;
            let ty = match wrap_attr {
                Some(WrapAttr::Skip) => quote!(#ty),
                Some(WrapAttr::GenericsOnly(_)) => wrap_generics_only(&ty),
                None => {
                    if is_type(
                        &ty,
                        &["Option", "std::option::Option", "core::option::Option"],
                    ) || is_type(&ty, &["Vec", "std::vec::Vec", "alloc::vec::Vec"])
                    {
                        wrap_generics_only(&ty)
                    } else if is_type(&ty, &["EnvField", "serde_env_field::EnvField"]) {
                        quote!(#ty)
                    } else {
                        quote!(::serde_env_field::EnvField<#ty>)
                    }
                }
            };

            let attrs = attrs_tokens(field.attrs);
            let vis = field.vis;
            let ident = field.ident;
            let colon = field.colon_token;

            quote! {
                #attrs
                #vis
                #ident
                #colon
                #ty
            }
        })
        .collect::<Punctuated<_, Token![,]>>()
        .to_token_stream()
}

fn process_variants(variants: impl Iterator<Item = syn::Variant>) -> TokenStream2 {
    variants
        .map(|mut variant| {
            let wrap_attr = take_env_field_wrap_attr(&mut variant.attrs);
            let fields = variant.fields;

            let fields = match wrap_attr {
                Some(WrapAttr::Skip) => quote!(#fields),
                Some(WrapAttr::GenericsOnly(span)) => abort!(
                    span,
                    "`generics_only` is supported only for fields, not for enum variants"
                ),
                None => wrap_fields(fields, WrapKind::Enum),
            };

            let attrs = attrs_tokens(variant.attrs);
            let ident = variant.ident;
            quote! {
                #attrs
                #ident
                #fields
            }
        })
        .collect::<Punctuated<_, Token![,]>>()
        .to_token_stream()
}

enum WrapKind {
    Struct,
    Enum,
}

fn wrap_fields(fields: syn::Fields, kind: WrapKind) -> TokenStream2 {
    let delim = match kind {
        WrapKind::Struct => quote!(;),
        WrapKind::Enum => quote!(),
    };

    match fields {
        syn::Fields::Named(fields) => {
            let fields = process_fields(fields.named.into_iter());
            quote![{
                #fields
            }]
        }
        syn::Fields::Unnamed(fields) => {
            let fields = process_fields(fields.unnamed.into_iter());
            quote![(#fields) #delim]
        }
        syn::Fields::Unit => delim,
    }
}

fn enum_env_field_wrap(data: syn::DataEnum) -> TokenStream2 {
    let variants = process_variants(data.variants.into_iter());
    quote! {{
        #variants
    }}
}
