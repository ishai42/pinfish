use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;

#[macro_use]
extern crate quote;
extern crate syn;

#[proc_macro_derive(PackTo, attributes(xdr))]
pub fn derive_pack_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen = impl_pack_to(&ast);
    gen.into()
}

#[proc_macro_derive(VecPackUnpack)]
pub fn derive_vec_pack_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = ast.ident;
    let span = Span::call_site();
    let gen = quote_spanned!( span =>
                              #[automatically_derived]
                              impl VecPackUnpack for #name {}
    );

    gen.into()
}

#[proc_macro_derive(UnpackFrom, attributes(xdr))]
pub fn derive_unpack_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen = impl_unpack_from(&ast);
    gen.into()
}

/// Transform the input into a token stream containing any generated implementations,
/// as well as all errors that occurred.
fn impl_pack_to(input: &syn::DeriveInput) -> TokenStream {
    let mut errors: Vec<syn::Error> = Vec::new();

    let mut output_tokens = match &input.data {
        syn::Data::Struct(ds) => impl_pack_to_struct(&input.ident, ds, &mut errors),
        syn::Data::Enum(de) => impl_pack_to_enum(&input.ident, de, &mut errors),
        syn::Data::Union(_) => {
            errors.push(syn::Error::new(
                input.span(),
                "`#[derive(PackTo)]` cannot be applied to unions",
            ));
            TokenStream::new()
        }
    };

    // Emit errors
    output_tokens.extend(errors.iter().map(|err| err.to_compile_error()));

    output_tokens
}

fn impl_pack_to_struct(
    name: &syn::Ident,
    ds: &syn::DataStruct,
    errors: &mut Vec<syn::Error>,
) -> TokenStream {
    let empty;
    let fields = match &ds.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(_) => {
            errors.push(syn::Error::new(
                ds.struct_token.span(),
                "`#![derive(PackTo)]` is not currently supported on tuple structs",
            ));

            return TokenStream::new();
        }

        syn::Fields::Unit => {
            empty = syn::FieldsNamed {
                brace_token: syn::token::Brace {
                    span: Span::call_site(),
                },
                named: syn::punctuated::Punctuated::new(),
            };
            &empty
        }
    };

    let fields: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let span = Span::call_site();
    return quote_spanned! { span =>
                            #[automatically_derived]
                            impl<B: xdr::Packer> xdr::PackTo<B> for #name {
                                fn pack_to(&self, buf: &mut B) {
                                    #(
                                        self.#fields . pack_to(buf);
                                        println!("packing {:?}", self.#fields);
                                    )*
                                }
                            }
    };
}

fn impl_pack_to_enum(
    name: &syn::Ident,
    de: &syn::DataEnum,
    errors: &mut Vec<syn::Error>,
) -> TokenStream {
    let mut arms = Vec::new();
    let mut discriminant = quote!(0);
    let mut has_discriminants = false;

    for variant in de.variants.iter() {
        let n_from_attr = discriminant_from_attr(errors, &variant.attrs);
        let n_from_discriminant = match &variant.discriminant {
            None => None,
            Some((_, expr)) => {
                has_discriminants = true;
                Some(expr.to_token_stream())
            }
        };

        if n_from_attr.is_some() && has_discriminants {
            errors.push(syn::Error::new(
                de.enum_token.span(),
                "`#![derive(PackTo)]` cannot mix custom discriminant and attribute based discriminant",
            ));

            continue;
        }

        discriminant = n_from_attr
            .or(n_from_discriminant.or(Some(discriminant)))
            .unwrap();

        let var_name = &variant.ident;
        let span = variant.span();

        let pack_inner;
        let capture;
        match &variant.fields {
            syn::Fields::Unit => {
                pack_inner = TokenStream::new();
                capture = TokenStream::new();
            }
            syn::Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() != 1 {
                    errors.push(syn::Error::new(
                        unnamed.paren_token.span,
                        "`#![derive(PackTo)]` enum variant cannot contain more than one field",
                    ));

                    continue;
                }
                capture = quote_spanned!( span => (inner) );
                pack_inner = quote_spanned!( span => inner.pack_to(buf); )
            }

            syn::Fields::Named(named) => {
                errors.push(syn::Error::new(
                    named.brace_token.span,
                    "`#![derive(PackTo)]` is not supported on tuple structs",
                ));

                continue;
            }
        };

        arms.push(quote_spanned! { span => #name::#var_name #capture => {
            buf.pack_uint(#discriminant);
            #pack_inner
        }, });

        discriminant = quote!((#discriminant)+1);
    }

    if arms.len() == 0 {
        errors.push(syn::Error::new(
            de.brace_token.span,
            "`#![derive(PackTo)]` cannot derive for empty enum",
        ));

        return TokenStream::new();
    }

    let span = de.brace_token.span;
    return quote_spanned! { span =>
                            #[automatically_derived]
                            impl<B: xdr::Packer> xdr::PackTo<B> for #name {
                                fn pack_to(&self, buf: &mut B) {
                                    match self {
                                    #(
                                        #arms
                                    )*
                                    }
                                }
                            }
    };
}

fn discriminant_from_attr(
    errors: &mut Vec<syn::Error>,
    attrs: &Vec<syn::Attribute>,
) -> Option<TokenStream> {
    for attr in attrs {
        let segments = &attr.path.segments;
        if segments.len() != 1 || segments[0].ident != "xdr" {
            continue;
        }
        return match attr.parse_args::<syn::Expr>() {
            Ok(expr) => Some(expr.to_token_stream()),
            Err(e) => {
                errors.push(e);
                None
            }
        };
    }

    None
}

/// Transform the input into a token stream containing any generated implementations,
/// as well as all errors that occurred.
fn impl_unpack_from(input: &syn::DeriveInput) -> TokenStream {
    let mut errors: Vec<syn::Error> = Vec::new();

    let mut output_tokens = match &input.data {
        syn::Data::Struct(ds) => impl_unpack_from_struct(&input.ident, ds, &mut errors),
        syn::Data::Enum(de) => impl_unpack_from_enum(&input.ident, de, &mut errors),
        syn::Data::Union(_) => {
            errors.push(syn::Error::new(
                input.span(),
                "`#[derive(PackTo)]` cannot be applied to unions",
            ));
            TokenStream::new()
        }
    };

    // Emit errors
    output_tokens.extend(errors.iter().map(|err| err.to_compile_error()));

    output_tokens
}

fn impl_unpack_from_struct(
    name: &syn::Ident,
    ds: &syn::DataStruct,
    errors: &mut Vec<syn::Error>,
) -> TokenStream {
    let empty;
    let fields = match &ds.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(_) => {
            errors.push(syn::Error::new(
                ds.struct_token.span(),
                "`#![derive(PackTo)]` is not currently supported on tuple structs",
            ));

            return TokenStream::new();
        }

        syn::Fields::Unit => {
            empty = syn::FieldsNamed {
                brace_token: syn::token::Brace {
                    span: Span::call_site(),
                },
                named: syn::punctuated::Punctuated::new(),
            };
            &empty
        }
    };

    let idents: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let types: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ty.to_token_stream())
        .collect();

    let span = Span::call_site();
    return quote_spanned! { span =>
                            #[automatically_derived]
                            impl<B: xdr::Unpacker> xdr::UnpackFrom<B> for #name {
                                fn unpack_from(buf: &mut B) -> Self {
                                    #name {
                                        #(
                                            #idents : <#types>::unpack_from(buf),
                                        )*
                                    }
                                }
                            }
    };
}

fn impl_unpack_from_enum(
    name: &syn::Ident,
    de: &syn::DataEnum,
    errors: &mut Vec<syn::Error>,
) -> TokenStream {
    let mut arms = Vec::new();
    let mut consts = Vec::new();
    let mut discriminant = quote!(0);
    let mut has_discriminants = false;
    let mut const_num : u32 = 0;

    for variant in de.variants.iter() {
        let n_from_attr = discriminant_from_attr(errors, &variant.attrs);
        let n_from_discriminant = match &variant.discriminant {
            None => None,
            Some((_, expr)) => {
                has_discriminants = true;
                Some(expr.to_token_stream())
            }
        };

        if n_from_attr.is_some() && has_discriminants {
            errors.push(syn::Error::new(
                de.enum_token.span(),
                "`#![derive(PackTo)]` cannot mix custom discriminant and attribute based discriminant",
            ));

            continue;
        }

        discriminant = n_from_attr
            .or(n_from_discriminant.or(Some(discriminant)))
            .unwrap();

        let var_name = &variant.ident;
        let span = variant.span();

        let unpack_inner;
        match &variant.fields {
            syn::Fields::Unit => {
                unpack_inner = TokenStream::new();
            }
            syn::Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() != 1 {
                    errors.push(syn::Error::new(
                        unnamed.paren_token.span,
                        "`#![derive(PackTo)]` enum variant cannot contain more than one field",
                    ));

                    continue;
                }
                let inner_ty = &unnamed.unnamed.first().as_ref().unwrap().ty;
                unpack_inner = quote_spanned!( span => ( <#inner_ty>::unpack_from(buf) ) );
            }

            syn::Fields::Named(named) => {
                errors.push(syn::Error::new(
                    named.brace_token.span,
                    "`#![derive(PackTo)]` is not supported on tuple structs",
                ));

                continue;
            }
        };

        let varname = format_ident!("_CONST{}", const_num);
        const_num += 1;
        consts.push(quote_spanned! { span => const #varname : u32 = #discriminant; });
        arms.push(quote_spanned! { span => #varname => { #name::#var_name #unpack_inner } });

        discriminant = quote!((#discriminant)+1);
    }

    if arms.len() == 0 {
        errors.push(syn::Error::new(
            de.brace_token.span,
            "`#![derive(PackTo)]` cannot derive for empty enum",
        ));

        return TokenStream::new();
    }

    let span = de.brace_token.span;
    return quote_spanned! { span =>
                            #[automatically_derived]
                            impl<B: xdr::Unpacker> xdr::UnpackFrom<B> for #name {
                                fn unpack_from(buf: &mut B) -> Self {
                                    #( #consts )*
                                    let n = buf.unpack_uint();
                                    match n {
                                    #(
                                        #arms
                                    )*
                                    _ => todo!("handle errors")
                                    }
                                }
                            }
    };
}
