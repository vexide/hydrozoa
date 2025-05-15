use core::convert::Into;
use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{ToTokens, quote, quote_spanned};
use serialize::SdkModule;
use syn::{
    DeriveInput, Expr, FnArg, Ident, LitInt, ReturnType, Token, Type, Variadic, braced,
    parenthesized,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
};

mod serialize;

/// Register a set of VEX SDK functions such that they can be accessed from the given
/// WASM `instance` using the given `store` by importing them from the specified
/// module name.
///
/// For example, the following code will create a function in the WASM module `module_name`
/// named `my_func` which calls `vex_sdk::my_func` and returns a WASM I32.
///
/// ```
/// link!(instance, store, mod "module_name" {
///     fn my_func() -> i32;
/// });
/// ```
///
/// You can optionally specify wrapper expressions on arguments to aid in conversion from the raw
/// WASM type (limited to i32, i64, f32, f64, v128, funcref, externref) to something `vex_sdk`
/// expects.
///
/// ```
/// link!(instance, store, mod "module_name" {
///     // Wraps `vex_sdk::my_func_raw(a: i32)`
///     fn my_func_raw(a: i32);
///
///     // Wraps `vex_sdk::my_func(a: ControllerId)`
///     fn my_func(a: i32 as |x| ControllerId(x));
///
///     // Equivalent to above
///     fn my_func(a: i32 as ControllerId);
/// });
/// ```
///
/// The opposite can be done with return types to aid in conversion back to a raw WASM type.
///
/// ```
/// link!(instance, store, mod "module_name" {
///     // Wraps `vex_sdk::my_func_raw() -> i32`
///     fn my_func_raw() -> i32;
///
///     // Wraps `vex_sdk::my_func() -> ControllerId`
///     fn my_func() -> i32 as |id| id.0;
/// });
/// ```
///
/// You can also specify a function as `printf fn` to automatically add an extra C-string parameter to the WASM function
/// which is passed to the underlying `vex_sdk` call using the `"%s"` format specifier.
#[proc_macro_error]
#[proc_macro]
pub fn link(input: TokenStream) -> TokenStream {
    let link_call = parse_macro_input!(input as LinkCall);

    let serialized = SdkModule::new(&link_call);
    if serialized.is_none() {
        return TokenStream::new();
    }

    let json = serde_json::to_vec_pretty(&serialized).unwrap();

    let out_file = PathBuf::from(env!("OUT_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("hydrozoa_api.json");
    std::fs::write(out_file, &json).unwrap();

    let LinkCall {
        instance_param,
        store_param,
        module_name,
        module_items,
    } = link_call;

    let mut item_tokens = vec![];
    for item in module_items {
        let LinkItem::Func(item) = item else {
            continue;
        };

        if item.declareness.is_some() {
            continue;
        }

        let name = item.ident;

        let mut args = vec![];
        let mut types = vec![];
        let mut arg_wrappers = vec![];

        for input in item.inputs {
            let arg;
            if let FnArg::Typed(inner) = input.fn_arg {
                arg = inner;
            } else {
                abort!(input.fn_arg, "`self` arguments aren't supported");
            }

            let raw_type;
            let wrapper_type;
            if let WrapperType::Convert(inner) = input.wrapper {
                raw_type = arg.ty;
                wrapper_type = Some(inner);
            } else {
                raw_type = arg.ty;
                wrapper_type = None;
            }

            types.push(raw_type);
            args.push(arg.pat);
            arg_wrappers.push(wrapper_type);
        }

        let is_printf = item.printfness.is_some();
        let mut printf_args = quote! {};
        let mut format_param = quote! {};
        let mut printf_convert = quote! {};
        if is_printf {
            printf_args = quote! { string, };
            format_param = quote! { c"%s".as_ptr(), string.as_ptr(), };
            printf_convert = quote! { let string = get_cstring(&mut ctx, string); };

            let string_type = syn::parse2(quote! { i32 }).unwrap();
            types.push(Box::new(string_type));
        }

        let mut return_wrapper = quote! {};
        let mut return_type = quote! { () };
        if let LinkItemReturnType::Type {
            return_type: inner,
            wrapper,
            ..
        } = item.output
        {
            if let WrapperType::Convert(wrapper) = wrapper {
                let span = wrapper.span();
                return_wrapper = quote_spanned! {span=> (|x: #wrapper| x.0)};
            }
            return_type = inner.to_token_stream();
        }

        let span = name.span();

        item_tokens.push(quote_spanned! {span=>
            #instance_param.link_closure(
                &mut * #store_param,
                #module_name,
                stringify!(#name),
                #[allow(unused_parens, unused, clippy::double_parens, clippy::redundant_closure_call)]
                |mut ctx, (#(#args,)* #printf_args): (#(#types,)*)| {
                    #printf_convert
                    let res: #return_type = unsafe {
                        #return_wrapper (vex_sdk::#name(
                            #(#arg_wrappers (#args as _),)*
                            #format_param
                        )) as _
                    };

                    Ok(res)
                }
            )?;
        });
    }

    quote! {
        #(#item_tokens)*
    }
    .into()
}

mod kw {
    syn::custom_keyword!(printf);
    syn::custom_keyword!(declare);
}

struct LinkCall {
    instance_param: Expr,
    store_param: Expr,
    module_name: syn::LitStr,
    module_items: Vec<LinkItem>,
}

impl Parse for LinkCall {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let instance = input.parse()?;
        input.parse::<Token![,]>()?;

        let store = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Token![mod]>()?;
        let module_name = input.parse()?;

        let content;
        braced!(content in input);

        let mut items = vec![];
        while !content.is_empty() {
            let item = content.parse()?;
            items.push(item);
        }

        Ok(Self {
            instance_param: instance,
            store_param: store,
            module_name,
            module_items: items,
        })
    }
}

enum LinkItem {
    Func(LinkFunc),
    Enum(LinkEnum),
}

impl Parse for LinkItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![enum]) {
            Ok(Self::Enum(input.parse()?))
        } else if lookahead.peek(Token![fn])
            || lookahead.peek(kw::printf)
            || lookahead.peek(kw::declare)
        {
            Ok(Self::Func(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

struct LinkFunc {
    declareness: Option<kw::declare>,
    printfness: Option<kw::printf>,
    fn_token: Token![fn],
    ident: Ident,
    paren_token: Paren,
    inputs: Punctuated<LinkItemArg, Token![,]>,
    variadic: Option<Token![...]>,
    output: LinkItemReturnType,
}

impl Parse for LinkFunc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut declareness = None;
        let mut printfness = None;

        let fn_token;
        loop {
            // OK: declare printf fn
            // OK: declare fn
            // OK: fn
            // ERR: fn printf
            // ERR: printf declare fn

            let lookahead = input.lookahead1();

            if lookahead.peek(Token![fn]) {
                fn_token = input.parse()?;
                break;
            }

            if printfness.is_none() && lookahead.peek(kw::printf) {
                printfness = Some(input.parse()?);
                continue;
            }

            if declareness.is_none() && lookahead.peek(kw::declare) {
                declareness = Some(input.parse()?);
                continue;
            }

            return Err(lookahead.error());
        }

        let ident = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);

        let mut inputs = Punctuated::new();
        let mut variadic = None;

        while !content.is_empty() {
            if let Ok(token) = content.parse::<Token![...]>() {
                variadic = Some(token);
                break;
            }

            let arg = content.parse()?;
            inputs.push_value(arg);

            if content.is_empty() {
                break;
            }

            let comma: Token![,] = content.parse()?;
            inputs.push_punct(comma);
        }

        let output = input.parse()?;

        input.parse::<Token![;]>()?;

        Ok(Self {
            declareness,
            printfness,
            fn_token,
            ident,
            paren_token,
            inputs,
            variadic,
            output,
        })
    }
}

struct LinkItemArg {
    fn_arg: FnArg,
    wrapper: WrapperType,
}

impl Parse for LinkItemArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_arg = input.parse()?;
        let wrapper = input.parse()?;
        Ok(Self { fn_arg, wrapper })
    }
}

enum LinkItemReturnType {
    Default,
    Type {
        arrow: Token![->],
        return_type: Box<Type>,
        wrapper: WrapperType,
    },
}

impl Parse for LinkItemReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![->]) {
            let arrow = input.parse()?;
            let return_type = Box::new(input.parse()?);
            let wrapper = input.parse()?;

            Ok(Self::Type {
                arrow,
                return_type,
                wrapper,
            })
        } else {
            Ok(Self::Default)
        }
    }
}

enum WrapperType {
    None,
    Convert(Box<Type>),
}

impl Parse for WrapperType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let wrapper = input.parse()?;

            Ok(Self::Convert(Box::new(wrapper)))
        } else {
            Ok(Self::None)
        }
    }
}

struct LinkEnum {
    name: Ident,
    underlying_type: Type,
    brace_token: Brace,
    variants: Punctuated<(Ident, LitInt), Token![,]>,
}

impl Parse for LinkEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let underlying_type = input.parse()?;

        let content;
        let brace = braced!(content in input);

        let mut variants = Punctuated::new();
        while !content.is_empty() {
            let variant = content.parse()?;
            content.parse::<Token![=]>()?;
            let value = content.parse()?;

            variants.push_value((variant, value));

            if content.is_empty() {
                break;
            }

            let comma: Token![,] = content.parse()?;
            variants.push_punct(comma);
        }

        Ok(Self {
            name,
            underlying_type,
            brace_token: brace,
            variants,
        })
    }
}
