//! Proc macros for `karpal-verify`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Item, ItemImpl, Lit, Meta, Type, parse_macro_input};

#[derive(Debug, Default)]
struct ExportConfig {
    crate_name: Option<String>,
    item_path: Option<String>,
    carrier: Option<String>,
    semigroup_op: Option<String>,
    monoid_op: Option<String>,
    monoid_identity: Option<String>,
    group_op: Option<String>,
    group_identity: Option<String>,
    group_inverse: Option<String>,
    semiring_add: Option<String>,
    semiring_zero: Option<String>,
    semiring_mul: Option<String>,
    semiring_one: Option<String>,
    ring_add: Option<String>,
    ring_zero: Option<String>,
    ring_neg: Option<String>,
    ring_mul: Option<String>,
    ring_one: Option<String>,
    lattice_meet: Option<String>,
    lattice_join: Option<String>,
}

/// Export backend-agnostic verification obligations for an annotated item.
#[proc_macro_attribute]
pub fn export_obligations(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_export_config(
        parse_macro_input!(attr with syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated),
    );
    let item = parse_macro_input!(item as Item);

    let Item::Impl(item_impl) = item else {
        return syn::Error::new_spanned(
            item,
            "#[export_obligations] currently supports inherent impl blocks",
        )
        .to_compile_error()
        .into();
    };

    expand_impl(config, item_impl).into()
}

fn expand_impl(config: ExportConfig, item_impl: ItemImpl) -> proc_macro2::TokenStream {
    let Some(type_ident) = impl_type_ident(&item_impl) else {
        return syn::Error::new_spanned(
            item_impl.self_ty.as_ref(),
            "#[export_obligations] requires a concrete type name",
        )
        .to_compile_error();
    };

    let crate_name = config.crate_name.unwrap_or_else(|| "unknown".to_string());
    let item_path = config.item_path.unwrap_or_else(|| type_ident.to_string());
    let bundle_name = item_path.clone();
    let carrier = sort_expr(config.carrier.as_deref().unwrap_or("Named"), &item_path);

    let bundle_expr = if let Some(op) = config.group_op {
        let identity = config
            .group_identity
            .unwrap_or_else(|| panic!("group requires identity = \"...\""));
        let inverse = config
            .group_inverse
            .unwrap_or_else(|| panic!("group requires inverse = \"...\""));
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::group(#carrier, #op, #identity, #inverse);
                karpal_verify::ObligationBundle::group(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else if let Some(add) = config.ring_add {
        let zero = config
            .ring_zero
            .unwrap_or_else(|| panic!("ring requires zero = \"...\""));
        let neg = config
            .ring_neg
            .unwrap_or_else(|| panic!("ring requires neg = \"...\""));
        let mul = config
            .ring_mul
            .unwrap_or_else(|| panic!("ring requires mul = \"...\""));
        let one = config
            .ring_one
            .unwrap_or_else(|| panic!("ring requires one = \"...\""));
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::ring(#carrier, #add, #mul, #zero, #one, #neg);
                karpal_verify::ObligationBundle::ring(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else if let Some(add) = config.semiring_add {
        let zero = config
            .semiring_zero
            .unwrap_or_else(|| panic!("semiring requires zero = \"...\""));
        let mul = config
            .semiring_mul
            .unwrap_or_else(|| panic!("semiring requires mul = \"...\""));
        let one = config
            .semiring_one
            .unwrap_or_else(|| panic!("semiring requires one = \"...\""));
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::semiring(#carrier, #add, #mul, #zero, #one);
                karpal_verify::ObligationBundle::semiring(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else if let Some(meet) = config.lattice_meet {
        let join = config
            .lattice_join
            .unwrap_or_else(|| panic!("lattice requires join = \"...\""));
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::lattice(#carrier, #meet, #join);
                karpal_verify::ObligationBundle::lattice(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else if let Some(op) = config.monoid_op {
        let identity = config
            .monoid_identity
            .unwrap_or_else(|| panic!("monoid requires identity = \"...\""));
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::monoid(#carrier, #op, #identity);
                karpal_verify::ObligationBundle::monoid(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else if let Some(op) = config.semigroup_op {
        quote! {
            {
                let signature = karpal_verify::AlgebraicSignature::semigroup(#carrier, #op);
                karpal_verify::ObligationBundle::semigroup(
                    #bundle_name,
                    karpal_verify::Origin::new(#crate_name, #item_path),
                    &signature,
                )
            }
        }
    } else {
        return syn::Error::new_spanned(
            item_impl.impl_token,
            "expected semigroup(...) or monoid(...) obligation family",
        )
        .to_compile_error();
    };

    quote! {
        #item_impl

        impl #type_ident {
            pub fn karpal_obligation_bundle() -> karpal_verify::ObligationBundle {
                #bundle_expr
            }
        }
    }
}

fn impl_type_ident(item_impl: &ItemImpl) -> Option<&syn::Ident> {
    match item_impl.self_ty.as_ref() {
        Type::Path(path) if path.qself.is_none() => path.path.get_ident(),
        _ => None,
    }
}

fn sort_expr(carrier: &str, item_path: &str) -> proc_macro2::TokenStream {
    match carrier {
        "Bool" => quote! { karpal_verify::Sort::Bool },
        "Int" => quote! { karpal_verify::Sort::Int },
        "Real" => quote! { karpal_verify::Sort::Real },
        "Named" => quote! { karpal_verify::Sort::named(#item_path) },
        other => quote! { karpal_verify::Sort::named(#other) },
    }
}

fn parse_export_config(items: syn::punctuated::Punctuated<Meta, syn::Token![,]>) -> ExportConfig {
    let mut config = ExportConfig::default();

    for meta in items {
        match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("crate_name") => {
                config.crate_name = string_lit(&name_value.value);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("item_path") => {
                config.item_path = string_lit(&name_value.value);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("carrier") => {
                config.carrier = string_lit(&name_value.value);
            }
            Meta::List(list) if list.path.is_ident("semigroup") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.semigroup_op = find_string_arg(&nested, "op");
            }
            Meta::List(list) if list.path.is_ident("monoid") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.monoid_op = find_string_arg(&nested, "op");
                config.monoid_identity = find_string_arg(&nested, "identity");
            }
            Meta::List(list) if list.path.is_ident("group") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.group_op = find_string_arg(&nested, "op");
                config.group_identity = find_string_arg(&nested, "identity");
                config.group_inverse = find_string_arg(&nested, "inverse");
            }
            Meta::List(list) if list.path.is_ident("semiring") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.semiring_add = find_string_arg(&nested, "add");
                config.semiring_zero = find_string_arg(&nested, "zero");
                config.semiring_mul = find_string_arg(&nested, "mul");
                config.semiring_one = find_string_arg(&nested, "one");
            }
            Meta::List(list) if list.path.is_ident("ring") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.ring_add = find_string_arg(&nested, "add");
                config.ring_zero = find_string_arg(&nested, "zero");
                config.ring_neg = find_string_arg(&nested, "neg");
                config.ring_mul = find_string_arg(&nested, "mul");
                config.ring_one = find_string_arg(&nested, "one");
            }
            Meta::List(list) if list.path.is_ident("lattice") => {
                let nested = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap_or_default();
                config.lattice_meet = find_string_arg(&nested, "meet");
                config.lattice_join = find_string_arg(&nested, "join");
            }
            _ => {}
        }
    }

    config
}

fn find_string_arg(
    items: &syn::punctuated::Punctuated<Meta, syn::Token![,]>,
    key: &str,
) -> Option<String> {
    items.iter().find_map(|meta| match meta {
        Meta::NameValue(name_value) if name_value.path.is_ident(key) => {
            string_lit(&name_value.value)
        }
        _ => None,
    })
}

fn string_lit(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => Some(value.value()),
            _ => None,
        },
        _ => None,
    }
}
