use std::collections::HashMap;

use proc_macro2;
use syn::spanned::Spanned;

const FINAL_BUILDER_SUFFIX: &str = "FinalBuilder";
const FIELDS_CONTAINER_SUFFIX: &str = "FieldsContainer";

/// Given a reference to a `syn::Type`, this function attempts to extract the inner type `T`
/// if the input type is an `Option<T>`. If the input type is not an `Option`, the function
/// returns `None`.
///
/// This extraction process is intricate because, during code generation, we lack information about the types.
/// Consequently, we must operate with token streams.
fn extract_from_option_type(ty: &syn::Type) -> Option<syn::Type> {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => segments
            .iter()
            .find(|segment| segment.ident == stringify!(Option))
            .map(|segment| match segment.arguments {
                syn::PathArguments::AngleBracketed(ref inner) => inner
                    .args
                    .first()
                    .map(|ty| match ty {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    })
                    .flatten(),
                _ => None,
            })
            .flatten(),
        _ => None,
    }
}

fn generate_maybe_wrapped_with_option(ty: &syn::Type) -> proc_macro2::TokenStream {
    if extract_from_option_type(ty).is_some() {
        return quote::quote!(#ty);
    }
    quote::quote!(::std::option::Option<#ty>)
}

fn generate_container_fields(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let wrapped_fields = data.fields.iter().map(|field| {
        let field_name = field.ident.clone();
        let wrapped_type = generate_maybe_wrapped_with_option(&field.ty);
        quote::quote! {
            #field_name: #wrapped_type
        }
    });
    quote::quote!(
        #(#wrapped_fields),*
    )
}

fn generate_container(
    struct_name: &syn::Ident,
    data: &syn::DataStruct,
) -> (syn::Ident, proc_macro2::TokenStream) {
    let fields = generate_container_fields(data);
    let builder_name = syn::Ident::new(
        &format!(
            "__{name}{FIELDS_CONTAINER_SUFFIX}",
            name = struct_name.to_string()
        ),
        struct_name.span(),
    );
    (
        builder_name.clone(),
        quote::quote!(
            #[derive(Default)]
            struct #builder_name {
                #fields
            }
        ),
    )
}

fn generate_final_builder(
    struct_name: &syn::Ident,
    builder_name: &syn::Ident,
    shared_builder_name: &syn::Ident,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let final_builder_fields = data.fields.iter().cloned().map(|field| {
        let name = field.ident.unwrap();
        if extract_from_option_type(&field.ty).is_some() {
            return quote::quote!(
                #name: self.shared.#name
            );
        }
        quote::quote!(
            #name: self.shared.#name.unwrap()
        )
    });
    quote::quote!(
        struct #builder_name {
            shared: #shared_builder_name
        }
        impl #builder_name {
            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#final_builder_fields),*
                }
            }
        }
    )
}

fn forge_cache_key(fields: &[syn::Field]) -> String {
    fields
        .iter()
        .cloned()
        .map(|field| field.ident.unwrap().to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn generate_builder(
    struct_name: &syn::Ident,
    fields_container_name: &syn::Ident,
    fields: Vec<syn::Field>,
    build_method_impl: &proc_macro2::TokenStream,
    include_build_method_impl: bool,
    generator_cache: &mut HashMap<String, (syn::Ident, proc_macro2::TokenStream)>,
) -> syn::Ident {
    let cache_key = forge_cache_key(fields.as_slice());
    if let Some((ident, _)) = generator_cache.get(&cache_key) {
        return ident.clone();
    }

    let builder_name = syn::Ident::new(
        &format!(
            "__{name}_{nonce}",
            name = struct_name.to_string(),
            nonce = uuid::Uuid::new_v4().to_string().replace("-", "")
        ),
        struct_name.span(),
    );

    let builder_methods = fields.iter().enumerate().map(|(index, field)| {
        let field_name = field.ident.clone();

        let mut new_fields = fields.clone();
        new_fields.remove(index);

        // Literally nothing else to generate, return to the FinalBuilder
        let next_builder_name = if new_fields.is_empty() {
            syn::Ident::new(
                &format!(
                    "__{name}{FINAL_BUILDER_SUFFIX}",
                    name = struct_name.to_string()
                ),
                struct_name.span(),
            )
        }
        // leftovers fields are all optional, should treat them differently
        else if new_fields
            .iter()
            .all(|field| extract_from_option_type(&field.ty).is_some())
        {
            generate_builder(
                struct_name,
                fields_container_name,
                new_fields,
                build_method_impl,
                true,
                generator_cache,
            )
        }
        // General case
        else {
            generate_builder(
                struct_name,
                fields_container_name,
                new_fields,
                build_method_impl,
                include_build_method_impl,
                generator_cache,
            )
        };

        let method_name = syn::Ident::new(
            &format!(
                "with_{name}",
                name = field.ident.clone().unwrap().to_string()
            ),
            field.ident.span(),
        );

        let field_type = extract_from_option_type(&field.ty).unwrap_or_else(|| field.ty.clone());
        quote::quote!(
            pub fn #method_name(mut self, value: impl Into<#field_type>) -> #next_builder_name {
                self.shared.#field_name = Some(value.into());
                #next_builder_name {shared: self.shared}
            }
        )
    });

    let build_method = if include_build_method_impl {
        build_method_impl.clone()
    } else {
        quote::quote!()
    };

    let builder_code = quote::quote!(
        #[allow(non_camel_case_types)]
        struct #builder_name {
            shared: #fields_container_name
        }

        impl #builder_name {
            #(#builder_methods)*
            #build_method
        }
    );

    generator_cache.insert(cache_key, (builder_name.clone(), builder_code));
    builder_name
}

#[proc_macro_derive(Builder)]
pub fn nicer_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);

    // We support structs only
    let syn::Data::Struct(data) = parsed.data else {
        panic!("This macro can only operate on structs")
    };

    // And the struct must have named fields, anon are no good
    if !data.fields.iter().all(|field| field.ident.is_some()) {
        panic!("This struct contains anon fields, which is not supported");
    }

    let (shared_builder_name, shared_builder_definition) = generate_container(&parsed.ident, &data);
    let final_builder_name = syn::Ident::new(
        &format!(
            "__{name}{FINAL_BUILDER_SUFFIX}",
            name = parsed.ident.to_string()
        ),
        parsed.ident.span(),
    );

    // That builder will contain the actual `build` method
    let final_builder = generate_final_builder(
        &parsed.ident,
        &final_builder_name,
        &shared_builder_name,
        &data,
    );

    let mut generator_cache = HashMap::with_capacity(1 + 2usize.pow((data.fields.len()) as u32));
    let struct_name = parsed.ident;

    let build_method_impl = {
        let builder_fields = data.fields.iter().map(|field| {
            let name = field.ident.clone().unwrap();
            if extract_from_option_type(&field.ty).is_some() {
                return quote::quote!(
                    #name: self.shared.#name
                );
            }
            quote::quote!(
                #name: self.shared.#name.unwrap()
            )
        });
        quote::quote!(
            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#builder_fields),*
                }
            }
        )
    };

    // That builder you'd get by invoking the `builder` method on the target struct
    let initial_builder_name = generate_builder(
        &struct_name,
        &shared_builder_name,
        data.fields.into_iter().collect(),
        &build_method_impl,
        false,
        &mut generator_cache,
    );

    // Recover builders implementations from the the generator cache
    let builders = generator_cache.into_values().map(|(_, tokens)| tokens);

    quote::quote!(
        #shared_builder_definition
        #(#builders)*
        #final_builder
        impl #struct_name {
            pub fn builder() -> #initial_builder_name {
                #initial_builder_name {
                    shared: #shared_builder_name::default()
                }
            }
        }
    )
    .into()
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use rstest::rstest;
    use syn::{parse_quote, Type};

    #[rstest(
        input_type,
        expected_output_str,
        case(parse_quote! { Option<i32> }, Some("i32")),
        case(parse_quote! { Result<Option<i32>, String> }, None),
        case(parse_quote! { Option<Option<i32>> }, Some("Option < i32 >")),
        case(parse_quote! { i32 }, None),
        case(parse_quote! { Vec<String> }, None)
    )]
    fn extract_from_option_type_test(input_type: Type, expected_output_str: Option<&str>) {
        let result = super::extract_from_option_type(&input_type);
        let result_str = result.map(|t| t.to_token_stream().to_string());
        let expected_output = expected_output_str.map(|s| s.to_string());
        assert_eq!(result_str, expected_output);
    }
}
