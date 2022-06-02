mod interface;
mod parser;

use std::{env::current_dir, fs};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse, parse_macro_input, parse_quote, FnArg, ItemImpl, LitStr};

use interface::{DataType, Identifier, ReturnType, Service, Struct};

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
    let internal = quote! { ::rusty_rpc_lib::internal_for_macro };
    let struct_name = to_syn_ident(struct_name);

    let struct_field_tokens: Vec<TokenStream> = struct_
        .fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_name = to_syn_ident(field_name);
            let type_token_stream = data_type_to_token_stream(field_type);
            quote! { pub #field_name: #type_token_stream, }
        })
        .collect();
    quote! {
        #[derive(::std::fmt::Debug, #internal::Serialize, #internal::Deserialize, ::std::clone::Clone)]
        pub struct #struct_name {
            #(#struct_field_tokens)*
        }
        impl #internal::RustyRpcStruct for #struct_name {
        }
    }
}

fn code_for_service(service_name: &Identifier, service: &Service) -> TokenStream {
    let internal = quote! { ::rusty_rpc_lib::internal_for_macro };
    let service_name = to_syn_ident(service_name);
    let service_proxy_name = format_ident!("{}_RustyRpcServiceProxy", service_name);

    let method_headers: Vec<TokenStream> = service
        .methods
        .iter()
        .map(|(method_name, method_type)| {
            let method_name = to_syn_ident(method_name);
            let non_self_params: Vec<FnArg> = method_type
                .non_self_params
                .iter()
                .map(|(param_name, param_type)| -> FnArg {
                    let param_name = to_syn_ident(param_name);
                    let param_type = data_type_to_token_stream(param_type);
                    parse_quote! { #param_name: #param_type }
                })
                .collect();
            let return_type = return_type_to_token_stream(&method_type.return_type);

            // Without the semicolon or {}
            quote! {
                fn #method_name(&self, #(#non_self_params),*) -> #return_type
            }
        })
        .collect();

    quote! {
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

            #(
                #method_headers ;
            )*
        }
        impl #internal::RustyRpcServiceClient for dyn #service_name {
            type ServiceProxy = #service_proxy_name;
        }

        /// ServiceProxy for #service_name
        pub struct #service_proxy_name {
            service_id: #internal::ServiceId,
            bytes_stream_sink: ::std::sync::Arc<::std::sync::Mutex<dyn #internal::ClientStreamSink>>,
        }
        impl #internal::RustyRpcServiceProxy<dyn #service_name> for #service_proxy_name {
            fn from_service_id(
                service_id: #internal::ServiceId,
                bytes_stream_sink: ::std::sync::Arc<::std::sync::Mutex<dyn #internal::ClientStreamSink>>,
            ) -> Self {
                Self { service_id, bytes_stream_sink }
            }
        }
        impl Drop for #service_proxy_name {
            fn drop(&mut self) {
                if std::thread::panicking() {
                    return;
                }
                todo!()
            }
        }
        impl #service_name for #service_proxy_name {
            #(
                #method_headers {
                    todo!()  // Serialize arguments and send to server
                }
            )*
        }
    }
}

fn to_syn_ident(ident: &Identifier) -> syn::Ident {
    syn::Ident::new(&ident.0, Span::call_site())
}

fn data_type_to_token_stream(type_: &DataType) -> TokenStream {
    match type_ {
        DataType::I32 => quote! { i32 },
        DataType::Struct(type_identifier) => {
            let temp = to_syn_ident(type_identifier);
            quote! { #temp }
        }
    }
}

fn return_type_to_token_stream(type_: &ReturnType) -> TokenStream {
    let inner_return_type = match type_ {
        ReturnType::ServiceRef(x) => {
            let internal = quote! { ::rusty_rpc_lib::internal_for_macro };
            let temp = to_syn_ident(x);
            quote! { #internal::ServiceRef<dyn #temp> }
        }
        ReturnType::Data(x) => data_type_to_token_stream(x),
    };
    quote! {
        ::std::io::Result<#inner_return_type>
    }
}
