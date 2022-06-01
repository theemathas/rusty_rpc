mod interface;
mod parser;

use std::{env::current_dir, fs};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::Parser;
use syn::{
    parse, parse_macro_input, parse_quote, Field, FnArg, ItemImpl, ItemStruct, ItemTrait, LitStr,
};

use interface::{Identifier, Service, Struct, Type};

use crate::parser::parse_interface;

macro_rules! my_compile_error {
    ($msg:expr) => {{
        return parse::Error::new(Span::call_site(), $msg)
            .into_compile_error()
            .into();
    }};
}

/// Macro to be used as a top-level item. It will create the traits and structs
/// corresponding to the items in the specified protocol file.
///
/// Example: `interface_file!("src/something.protocol");`
#[proc_macro]
pub fn interface_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let protocol_file_path = current_dir().unwrap().join(input.value());
    let interface_file_contents = match fs::read_to_string(&protocol_file_path) {
        Ok(s) => s,
        Err(_) => my_compile_error!("Unable to read the specified protocol file."),
    };
    let rpc_interface = match parse_interface(interface_file_contents.as_bytes()) {
        Ok((_, x)) => x,
        Err(e) => my_compile_error!(format!("Error parsing the interface file: {e}")),
    };

    let all_code_for_structs = rpc_interface
        .structs
        .iter()
        .map(|(x, y)| code_for_struct(x, y));
    let all_code_for_services = rpc_interface
        .services
        .iter()
        .map(|(x, y)| code_for_service(x, y));

    let path_str = protocol_file_path.to_str().unwrap();
    quote! {
        const _HACK_TO_FORCE_RECOMPILE_UPON_CHANGING_PROTOCOL_FILE: &'static str = include_str!(#path_str);
        #(#all_code_for_structs)*
        #(#all_code_for_services)*
    }
    .into()
}

/// Macro to be used on each service implementation.
///
/// Example:
/// ```ignore
/// // A service named MyService is defined in the protocol file elsewhere
/// // using the interface_file! macro
///
/// struct MyServiceImpl;
///
/// #[service_server_impl]
/// impl MyService for MyServiceImpl {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn service_server_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let original_input = TokenStream::from(input.clone());
    let input = parse_macro_input!(input as ItemImpl);

    let service_type_name = input.self_ty;
    let service_trait_name = match input.trait_ {
        Some((_, x, _)) => x,
        None => my_compile_error!(
            "#[service_server_impl] should only be used on service trait implementations."
        ),
    };

    let internal = quote! { ::rusty_rpc_lib::internal_for_macro };
    quote! {
        #original_input

        impl #internal::RustyRpcServiceServer for #service_type_name {
            fn parse_and_call_method_locally(
                &mut self,
                method_and_args: #internal::MethodAndArgs,
                connection: &mut #internal::ServiceCollection,
            ) -> ::std::io::Result<#internal::ServerMessage> {
                <#service_type_name as #service_trait_name>::_rusty_rpc_forward__parse_and_call_method_locally(
                    self,
                    method_and_args,
                    connection
                )
            }
        }
    }
    .into()
}

fn code_for_struct(struct_name: &Identifier, struct_: &Struct) -> TokenStream {
    let struct_name = to_syn_ident(struct_name);

    // TODO Do I derive Clone if there's a service inside?
    let mut struct_type: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_name {}
    };
    let struct_fields = match &mut struct_type.fields {
        syn::Fields::Named(x) => &mut x.named,
        _ => unreachable!(),
    };
    *struct_fields = struct_
        .fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_name = to_syn_ident(field_name);
            let type_token_stream = type_to_token_stream(field_type);
            Field::parse_named
                .parse2(quote! { pub #field_name: #type_token_stream })
                .unwrap()
        })
        .collect();

    let struct_impl: ItemImpl = parse_quote! {
        impl ::rusty_rpc_lib::internal_for_macro::RustyRpcStruct for #struct_name {
        }
    };

    quote! {
        #struct_type
        #struct_impl
    }
}

fn code_for_service(service_name: &Identifier, service: &Service) -> TokenStream {
    let internal = quote! { ::rusty_rpc_lib::internal_for_macro };
    let service_name = to_syn_ident(service_name);

    let method_headers: Vec<TokenStream> = service.methods.iter().map(|(method_name, method_type)| {
        let method_name = to_syn_ident(method_name);
        let non_self_params: Vec<FnArg> = method_type
            .non_self_params
            .iter()
            .map(|(param_name, param_type)| {
                let param_name = to_syn_ident(param_name);
                let param_type = type_to_token_stream(param_type);
                let temp: FnArg = parse_quote! { #param_name: #param_type };
                temp
            })
            .collect();
        let return_type = type_to_token_stream(&method_type.return_type);

        // Without the semicolon or {}
        quote! {
            fn #method_name(&self, #(#non_self_params),*) -> #internal::ServerResult<(#return_type)>
        }
    }).collect();

    let mut service_trait: ItemTrait = parse_quote! {
        pub trait #service_name {
            /// This method should be automatically implemented by using the `#[service_server_impl]` macro
            #[doc(hidden)]
            fn _rusty_rpc_forward__parse_and_call_method_locally(
                &mut self,
                method_and_args: #internal::MethodAndArgs,
                connection: &mut #internal::ServiceCollection,
            ) -> ::std::io::Result<#internal::ServerMessage> {
                todo!()
            }
        }
    };
    // Add methods corresponding to the protocol file.
    service_trait
        .items
        .extend(method_headers.iter().map(|header| {
            parse_quote! {
                #header ;
            }
        }));

    let mut server_response_impl: ItemImpl = parse_quote! {
        impl #service_name for #internal::ServerResponse<&dyn #service_name> {}
    };
    server_response_impl
        .items
        .extend(method_headers.iter().map(|header| {
            parse_quote! {
                #header {
                    todo!()  // Serialize arguments and send to server
                }
            }
        }));

    quote! {
        #service_trait
        #server_response_impl
        impl #internal::RustyRpcServiceClient for dyn #service_name {}
    }
}

fn to_syn_ident(ident: &Identifier) -> syn::Ident {
    syn::Ident::new(&ident.0, Span::call_site())
}

fn type_to_token_stream(type_: &Type) -> TokenStream {
    match type_ {
        interface::Type::I32 => quote! { i32 },
        interface::Type::Struct(type_identifier) => {
            let temp = to_syn_ident(type_identifier);
            quote! { #temp }
        }
    }
}
