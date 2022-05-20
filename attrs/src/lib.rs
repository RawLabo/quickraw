use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn bench(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);

    let name = match syn::parse::<syn::Ident>(attr) {
        Ok(x) => x.to_string(),
        Err(_) => sig.ident.to_string(),
    };

    let ts = quote! {
        #(#attrs)* #vis #sig {
            let __t = std::time::Instant::now();
            let __result = #block;
            match std::env::var(crate::BENCH_FLAG) {
                Ok(_) => {
                    println!("{}: {:?}", #name, __t.elapsed());
                },
                Err(_) => {}
            };
            __result
        }
    };
    ts.into()
}

#[proc_macro_attribute]
pub fn pause(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);
    let fn_name = sig.ident.to_string();

    let token_flush_pause = quote! {
        {
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
        std::io::stdin().read_line(&mut String::new()).unwrap();
    };
    let pause_print = quote! {
        print!("Press enter to start '{}'... ", #fn_name);
        #token_flush_pause
    };

    let pause_end = match syn::parse::<syn::Ident>(attr) {
        Ok(x) => {
            if &x.to_string() == "both" {
                quote! {
                    print!("Press enter to end '{}'... ", #fn_name);
                    #token_flush_pause
                }
            } else {
                quote! {}
            }
        }
        Err(_) => quote! {},
    };

    let ts = quote! {
        #(#attrs)* #vis #sig {
            #pause_print;
            let __result = #block;
            #pause_end;
            __result
        }
    };
    ts.into()
}
