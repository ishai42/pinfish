use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

#[macro_use]
extern crate quote;
extern crate syn;

#[proc_macro_derive(PackTo)]
pub fn derive_pack_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen = impl_pack_to(&ast);
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
    _errors: &mut Vec<syn::Error>,
) -> TokenStream {
    let span = de.brace_token.span;

    return quote_spanned! { span =>
                            #[automatically_derived]
                            impl<B: xdr::Packer> xdr::PackTo<B> for #name {
                                fn pack_to(&self, buf: &mut B) {
                                    //#(
                                        todo!();
                                    //)*
                                }
                            }
    };
}
