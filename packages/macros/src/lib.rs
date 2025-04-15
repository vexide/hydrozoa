use core::convert::Into;

use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{
    DeriveInput, Expr, FnArg, Ident, ReturnType, Token, Type, Variadic, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Paren,
};

#[proc_macro]
pub fn link(input: TokenStream) -> TokenStream {
    let LinkCall {
        instance_param,
        store_param,
        module_name,
        module_items,
    } = parse_macro_input!(input as LinkCall);

    let mut item_tokens = vec![];
    for item in module_items {
        let name = item.ident;
        item_tokens.push(quote! {
            #instance_param.link_closure(
                &mut * #store_param,
                #module_name,
                stringify!(#name),
                // #[allow(unused_parens)]
                // |mut ctx, ($($arg,)* string): ($($arg_ty,)* i32)| {
                //     let string = get_cstring(&mut ctx, string);
                //     unsafe {
                //         vex_sdk::$name(
                //             $($arg,)*
                //             c"%s".as_ptr(),
                //             string.as_ptr(),
                //         );
                //     }
                //     Ok(())
                // }
            )?;
        });
    }

    quote! {
        #(#item_tokens)*
    }.into()
}

mod kw {
    syn::custom_keyword!(printf);
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

            content.parse::<Token![;]>()?;
        }

        Ok(Self {
            instance_param: instance,
            store_param: store,
            module_name,
            module_items: items,
        })
    }
}

struct LinkItem {
    printfness: Option<kw::printf>,
    fn_token: Token![fn],
    ident: Ident,
    paren_token: Paren,
    inputs: Punctuated<FnArg, Token![,]>,
    variadic: Option<Token![...]>,
    output: ReturnType,
}

impl Parse for LinkItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let printfness = if input.peek(kw::printf) {
            Some(input.parse()?)
        } else {
            None
        };
        let fn_token = input.parse()?;
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

            let arg = input.parse()?;
            inputs.push_value(arg);

            if input.is_empty() {
                break;
            }

            let comma: Token![,] = input.parse()?;
            inputs.push_punct(comma);
        }

        let output = input.parse()?;

        Ok(Self {
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
    raw_type: RawType,
}

impl Parse for LinkItemArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_arg = input.parse()?;
        let raw_type = input.parse()?;
        Ok(Self { fn_arg, raw_type })
    }
}

enum LinkItemReturnType {
    Default,
    Type {
        arrow: Token![->],
        return_type: Box<Type>,
        raw_type: RawType,
    },
}

impl Parse for LinkItemReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![->]) {
            let arrow = input.parse()?;
            let return_type = Box::new(input.parse()?);
            let raw_type = input.parse()?;

            Ok(Self::Type {
                arrow,
                return_type,
                raw_type,
            })
        } else {
            Ok(Self::Default)
        }
    }
}

enum RawType {
    None,
    Type(Token![as], Box<Type>),
}

impl Parse for RawType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![as]) {
            let as_token = input.parse::<Token![as]>()?;
            let raw_type = input.parse()?;

            Ok(Self::Type(as_token, Box::new(raw_type)))
        } else {
            Ok(Self::None)
        }
    }
}
