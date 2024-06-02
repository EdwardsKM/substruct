extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DataStruct, DeriveInput,
    Field, Fields, FieldsNamed, Ident, MetaNameValue, Token,
};

/// Enables the generaation of structs that are a subset of the given struct.
/// #[sub_struct(name = "CreateCustomerParams", columns = ["id", "live_mode", "created", "updated", "delinquent"])]
#[proc_macro_attribute]
pub fn sub_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let parsed_args =
        parse_macro_input!(args with Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated);

    let meta_list = match parse_sub_struct_args(&parsed_args) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let new_struct = match ast.data {
        syn::Data::Struct(ref ds) => {
            match &ds.fields {
                Fields::Named(named_fields) => {
                    let mut unrecognized_fields = vec![];

                    // Validates that the provided fields actually exist.
                    match &meta_list.strategy {
                        Strategy::Remove(fs) => {
                            // Validate that the fields provided by the user actually exist on the ident
                            for field in fs {
                                if !named_fields
                                    .named
                                    .iter()
                                    .any(|f| f.ident.as_ref().unwrap().to_string() == *field)
                                {
                                    unrecognized_fields.push(field.clone());
                                }
                            }
                            // If one of the given fields to remove is not recognized, return an error
                            if !unrecognized_fields.is_empty() {
                                let invalid_fields_str = unrecognized_fields.join(", ");

                                return ::syn::Error::new_spanned(
                                    &ast.ident,
                                    format!(
                                        "Invalid field(s) specified in the remove attribute: {}",
                                        invalid_fields_str
                                    ),
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                        Strategy::Retain(fs) => {
                            // Validate that the fields provided by the user actually exist on the ident
                            for field in fs {
                                if !named_fields
                                    .named
                                    .iter()
                                    .any(|f| f.ident.as_ref().unwrap().to_string() == *field)
                                {
                                    unrecognized_fields.push(field.clone());
                                }
                            }
                            // If one of the given fields to remove is not recognized, return an error
                            if !unrecognized_fields.is_empty() {
                                let invalid_fields_str = unrecognized_fields.join(", ");

                                return ::syn::Error::new_spanned(
                                    &ast.ident,
                                    format!(
                                        "Invalid field(s) specified in the retain attribute: {}",
                                        invalid_fields_str
                                    ),
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                    };

                    // Generate a new struct

                    let new_fields: FieldsNamed = {
                        let fields: Punctuated<Field, Token![,]> = match &meta_list.strategy {
                            Strategy::Remove(fs) => named_fields
                                .named
                                .iter()
                                .filter(|f| !fs.contains(&f.ident.as_ref().unwrap().to_string()))
                                .cloned()
                                .collect(),
                            Strategy::Retain(fs) => named_fields
                                .named
                                .iter()
                                .filter(|f| fs.contains(&f.ident.as_ref().unwrap().to_string()))
                                .cloned()
                                .collect(),
                        };

                        FieldsNamed {
                            brace_token: syn::token::Brace::default(),
                            named: fields,
                        }
                    };

                    let new = Data::Struct(DataStruct {
                        struct_token: ds.struct_token.clone(),
                        fields: Fields::Named(new_fields),
                        semi_token: ds.semi_token.clone(),
                    });

                    let new_struct = DeriveInput {
                        data: new,
                        attrs: ast.attrs.clone(),
                        vis: ast.vis.clone(),
                        ident: Ident::new(&meta_list.name, Span::call_site()),
                        generics: ast.generics.clone(),
                    };

                    new_struct
                }
                _ => {
                    return ::syn::Error::new_spanned(
                        &ast.ident,
                        "sub_struct only supports structs with named fields",
                    )
                    .to_compile_error()
                    .into()
                }
            }
        }
        _ => {
            return ::syn::Error::new_spanned(
                &ast.ident,
                "sub_struct only supports structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    let final_output = quote! {
        #ast
        #new_struct
    };

    final_output.into()
}

// When generating a new struct from an existing struct, one can choose the fields to remove or fields to retain.
// These options are mutually exclusive.
struct SubStructAttributes {
    name: String,
    strategy: Strategy,
}

enum Strategy {
    Remove(Vec<String>),
    Retain(Vec<String>),
}

// Parse the Punctuated<MetaNameValue, Comma> into something usable.
fn parse_sub_struct_args(
    args: &syn::punctuated::Punctuated<MetaNameValue, syn::token::Comma>,
) -> Result<SubStructAttributes, syn::Error> {
    let mut name = String::new();
    let mut remove = vec![];
    let mut retain = vec![];

    for arg in args {
        if arg.path.is_ident("name") {
            match &arg.value {
                syn::Expr::Lit(v) => match &v.lit {
                    syn::Lit::Str(n) => {
                        name = n.value();
                    }
                    _ => {
                        return Err(syn::Error::new(
                            arg.span(),
                            "The name attribute only accepts strings as valid inputs",
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new(
                        arg.span(),
                        "Could not parse as a valid value for the name attribute",
                    ))
                }
            }
        } else if arg.path.is_ident("remove") {
            match &arg.value {
                syn::Expr::Array(v) => {
                    for e in v.elems.iter() {
                        match e {
                            syn::Expr::Lit(v) => match &v.lit {
                                syn::Lit::Str(n) => {
                                    remove.push(n.value())
                                },
                                _ => return Err(syn::Error::new(
                                    arg.span(),
                                    "Could not parse as a valid string. The name attribute only accepts strings as valid inputs",
                                )),
                            },
                            _ => return Err( syn::Error::new(
                                arg.span(),
                                "The remove attribute only accepts an array of strings as valid inputs",
                            )),
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.span(),
                        "The remove attribute only accepts an array strings as valid inputs",
                    ))
                }
            };
        } else if arg.path.is_ident("retain") {
            match &arg.value {
                syn::Expr::Array(v) => {
                    for e in v.elems.iter() {
                        match e {
                            syn::Expr::Lit(v) => match &v.lit {
                                syn::Lit::Str(n) => {
                                    retain.push(n.value())
                                },
                                _ => return Err(syn::Error::new(
                                    arg.span(),
                                    "Could not parse as a valid string. The name attribute only accepts strings as valid inputs",
                                )),
                            },
                            _ => return Err( syn::Error::new(
                                arg.span(),
                                "The remove attribute only accepts an array of strings as valid inputs",
                            )),
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.span(),
                        "The remove attribute only accepts an array strings as valid inputs",
                    ))
                }
            };
        } else {
            return Err(syn::Error::new(
                proc_macro::Span::call_site().into(),
                format!(
                "{} is not a valid attribute for sub_struct. Valid attributes are name and remove",
                arg.path.get_ident().unwrap().to_string()
            ),
            ));
        };
    }

    match name.is_empty() {
        false => match remove.is_empty() {
            // Under this branch, remove has been specified, so inclusion of retain is invalid
            false => match retain.is_empty() {
                false => Err(syn::Error::new(
                    proc_macro::Span::call_site().into(),
                    "Only one of remove or retain attributes can be used at a time",
                )),
                true => Ok(SubStructAttributes {
                    name,
                    strategy: Strategy::Remove(remove),
                }),
            },
            // If remove is_empty, then retain should be provided.
            true => match retain.is_empty() {
                true => Err(syn::Error::new(
                    proc_macro::Span::call_site().into(),
                    "Only one of remove or retain attributes can be used at a time",
                )),
                false => Ok(SubStructAttributes {
                    name,
                    strategy: Strategy::Retain(retain),
                }),
            },
        },
        true => Err(syn::Error::new(
            proc_macro::Span::call_site().into(),
            "The `name` attribute is required",
        )),
    }
}
