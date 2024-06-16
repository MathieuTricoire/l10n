use self::ast::{Field, Input};
use self::digest::{Digest, EnumDigest, Message, StructDigest};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, GenericArgument, GenericParam, Ident, Lifetime, LifetimeParam, Member,
    PathArguments, Result, Type, Visibility,
};

mod ast;
mod digest;

pub fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(&derive_input)?;
    Ok(match Digest::from_input(input)? {
        Digest::Struct(digest) => expand_struct(digest),
        Digest::Enum(digest) => expand_enum(digest),
    })
}

fn expand_struct(digest: StructDigest) -> TokenStream {
    let TraitData {
        impl_generics,
        impl_trait,
        ty,
        ty_generics,
        where_clause,
        l10n_self_lifetime,
        original_impl_generics,
    } = get_trait_data(digest.derive_input, digest.self_lifetime);
    let pat = fields_pat(&digest.fields);
    let translate_method_body = expand_translate_method_body(&digest.message, pat);
    let translate_method = quote! {
        fn try_translate_with_args(
            &#l10n_self_lifetime self,
            locale: &::l10n::unic_langid::LanguageIdentifier,
            args: std::option::Option<&#l10n_self_lifetime ::l10n::fluent_bundle::FluentArgs<#l10n_self_lifetime>>
        ) -> std::result::Result<std::borrow::Cow<'__l10n_result, str>, ::l10n::TranslateError> {
            #translate_method_body
        }
    };

    let from_impl = digest.from_field.map(|field| {
        let from = unoptional_type(field.ty);
        let from_member = &field.member;
        let some_source = if type_is_option(field.ty) {
            quote!(std::option::Option::Some(source))
        } else {
            quote!(source)
        };
        quote! {
            #[allow(unused_qualifications)]
            impl #original_impl_generics std::convert::From<#from> for #ty #ty_generics #where_clause {
                #[allow(deprecated)]
                fn from(source: #from) -> Self {
                    #ty { #from_member: #some_source }
                }
            }
        }
    });

    quote! {
        impl #impl_generics #impl_trait for #ty #ty_generics #where_clause {
            #translate_method
        }
        #from_impl
    }
}

fn expand_enum(digest: EnumDigest) -> TokenStream {
    let TraitData {
        impl_generics,
        impl_trait,
        ty,
        ty_generics,
        where_clause,
        l10n_self_lifetime,
        original_impl_generics,
    } = get_trait_data(digest.derive_input, digest.l10n_self_lifetime);

    let mut from_impls: Vec<TokenStream> = vec![];

    let variant_arms = digest.variants.into_iter().map(|variant| {
        let ident = &variant.variant_input.ident;
        if let Some(field) = variant.from_field {
            let from = unoptional_type(field.ty);
            let from_member = &field.member;
            let some_source = if type_is_option(field.ty) {
                quote!(std::option::Option::Some(source))
            } else {
                quote!(source)
            };
            from_impls.push(quote! {
                #[allow(unused_qualifications)]
                impl #original_impl_generics std::convert::From<#from> for #ty #ty_generics #where_clause {
                    #[allow(deprecated)]
                    fn from(source: #from) -> Self {
                        #ty::#ident { #from_member: #some_source }
                    }
                }
            });
        }

        let translate_method_body = expand_translate_method_body(&variant.message, None);
        let pat = fields_pat(&variant.fields);
        quote!(#ty::#ident #pat => { #translate_method_body },)
    });

    let translate_method = quote! {
        fn try_translate_with_args(
            &#l10n_self_lifetime self,
            locale: &::l10n::unic_langid::LanguageIdentifier,
            args: std::option::Option<&#l10n_self_lifetime ::l10n::fluent_bundle::FluentArgs<#l10n_self_lifetime>>
        ) -> std::result::Result<std::borrow::Cow<'__l10n_result, str>, ::l10n::TranslateError> {
            #[allow(unused_variables, clippy::used_underscore_binding)]
            match self {
                #(#variant_arms)*
            }
        }
    };

    quote! {
        impl #impl_generics #impl_trait for #ty #ty_generics #where_clause {
            #translate_method
        }
        #(#from_impls)*
    }
}

fn fields_pat(fields: &[Field]) -> Option<TokenStream> {
    if fields.is_empty() {
        return None;
    }
    let mut members = fields.iter().map(|field| &field.member).peekable();
    Some(match members.peek() {
        Some(Member::Named(_)) => quote!({ #(#members),* }),
        Some(Member::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                Member::Unnamed(index) => format_ident!("__self_{}", index),
                Member::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    })
}

pub fn field_to_ident(field: &Field) -> Ident {
    match &field.member {
        Member::Named(ident) => ident.clone(),
        Member::Unnamed(index) => format_ident!("__self_{}", index),
    }
}

struct TraitData {
    impl_generics: TokenStream,
    impl_trait: TokenStream,
    ty: TokenStream,
    ty_generics: TokenStream,
    where_clause: TokenStream,
    l10n_self_lifetime: Lifetime,
    original_impl_generics: TokenStream,
}

fn get_trait_data(original: &DeriveInput, l10n_self_lifetime: Option<Lifetime>) -> TraitData {
    let mut generics = original.generics.clone();

    let l10n_self_lifetime = l10n_self_lifetime.unwrap_or_else(|| {
        let lifetime = syn::Lifetime::new("'__l10n_self", Span::call_site());
        generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())));
        lifetime
    });

    generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(
            syn::Lifetime::new("'__l10n_result", Span::call_site()),
        )));

    let (original_impl_generics, ty_generics, where_clause) = original.generics.split_for_impl();
    let (impl_generics, _, _) = generics.split_for_impl();

    let impl_trait = spanned_impl_trait(original, &l10n_self_lifetime);
    let ty = &original.ident;

    TraitData {
        impl_generics: impl_generics.to_token_stream(),
        impl_trait,
        ty: ty.to_token_stream(),
        ty_generics: ty_generics.to_token_stream(),
        where_clause: where_clause.to_token_stream(),
        l10n_self_lifetime,
        original_impl_generics: original_impl_generics.to_token_stream(),
    }
}

fn spanned_impl_trait(input: &DeriveInput, args_lifetime: &Lifetime) -> TokenStream {
    let vis_span = match &input.vis {
        Visibility::Public(pub_token) => Some(pub_token.span()),
        Visibility::Restricted(vis) => Some(vis.pub_token.span()),
        Visibility::Inherited => None,
    };
    let data_span = match &input.data {
        Data::Struct(data) => data.struct_token.span(),
        Data::Enum(data) => data.enum_token.span(),
        Data::Union(data) => data.union_token.span(),
    };
    let first_span = vis_span.unwrap_or(data_span);
    let last_span = input.ident.span();
    let path = quote_spanned!(first_span=> ::l10n::);
    let impl_trait = quote_spanned!(last_span=> L10nMessage<#args_lifetime, '__l10n_result>);
    quote!(#path #impl_trait)
}

fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

fn unoptional_type(ty: &Type) -> TokenStream {
    let unoptional = type_parameter_of_option(ty).unwrap_or(ty);
    quote!(#unoptional)
}

fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}

fn expand_translate_method_body(l10n: &Message, pat: Option<TokenStream>) -> TokenStream {
    match l10n {
        Message::Transparent { field } => {
            let pat = pat.map(|pat| quote!(let Self #pat = self;));
            quote! {
                #pat
                #field.try_translate_with_args(locale, args)
            }
        }
        Message::Params {
            resource,
            key,
            arguments,
        } => {
            if arguments.is_empty() {
                quote!(crate::L10N.try_translate_with_args(locale, #resource, #key, args))
            } else {
                let local_args_set = arguments.iter().map(|arg| {
                    let name = arg.name();
                    let value = arg.value();
                    quote!(local_args.set(#name, #value);)
                });
                let set_local_args = if let Some(pat) = pat {
                    quote! {
                        {
                            let Self #pat = self;
                            #(#local_args_set)*
                        }
                    }
                } else {
                    quote!(#(#local_args_set)*)
                };
                let local_args = quote! {
                    let mut local_args = ::l10n::fluent_bundle::FluentArgs::new();
                    #set_local_args
                    if let std::option::Option::Some(args) = args {
                        for (key, value) in args.iter() {
                            local_args.set(key, value.to_owned());
                        }
                    }
                };

                quote!({
                    #local_args
                    crate::L10N.try_translate_with_args(locale, #resource, #key, std::option::Option::Some(&local_args))
                })
            }
        }
    }
}
