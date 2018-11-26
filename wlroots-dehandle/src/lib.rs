extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{Block, DeriveInput, ItemFn, Stmt, Expr, Pat, Local,
          parse::{self, Parse, ParseStream},
          punctuated::Punctuated,
          fold::Fold};

/// Parses a list of variable names separated by commas
///
///     a, b, c
///
/// This is how the compiler passes in arguments to our attribute -- it is
/// everything inside the delimiters after the attribute name.
///
///     #[wlroots_dehandle(a, b, c)]
///
struct Args {
    vars: HashSet<Ident>
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

impl Args {
    fn is_handle(&self, p: &Punctuated<Pat, Token![|]>) -> bool {
        if p.len() != 1 {
            return false;
        }
        match p[0] {
            Pat::Ident(ref p) => self.vars.contains(&p.ident),
            _ => false
        }
    }
}

/// Attribute to automatically call the `run` method on handles with the
/// remaining block of code.
///
/// # Example
///
/// ```
/// impl InputManagerHandler for InputManager {
///     #[wlroots_dehandle(compositor, keyboard, seat)]
///     fn keyboard_added(&mut self,
///                       compositor: CompositorHandle,
///                       keyboard: KeyboardHandle)
///                       -> Option<Box<Keyboard Handler>> {
///         let compositor = compositor;
///         let keyboard = keyboard;
///         //dehandle!(compositor, compositor);
///         //dehandle!(keyboard, keyboard);
///         let server: &mut ::Server = compositor.into();
///         server.keyboards.push(keyboard.weak_reference());
///         // Now that we have at least one keyboard, update the seat capabilities.
///         //dehandle!(seat, &server.seat.seat);
///         let seat = &server.seat.seat;
///         let mut capabilities = seat.capabilities();
///         capabilities.insert(Capability::Keyboard);
///         seat.set_capabilities(capabilities);
///         seat.set_keyboard(keyboard.input_device());
///         Some(Box::new(::Keyboard))
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn wlroots_dehandle(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as Args);
    let output = build_block(input.block.stmts.iter(), &args);
    input.block = parse_quote!({#(#output)*});
    TokenStream::from(quote!(#input))
}

fn build_block(mut input: std::slice::Iter<Stmt>, args: &Args) -> Vec<Stmt> {
    let mut output = vec![];
    let mut inner = None;
    while let Some(stmt) = input.next().cloned() {
        match stmt {
            Stmt::Local(local) => {
                if local.init.is_some() && args.is_handle(&local.pats) {
                    inner = Some(local.clone());
                    break;
                } else {
                    output.push(Stmt::Local(local.clone()));
                }
            }
            _ => output.push(stmt.clone())
        }
    }
    if let Some(inner) = inner {
        let (_, init_expr) = inner.init
            .expect("Let statement had no init expression");
        let var_name = inner.pats;
        let inner_output = build_block(input, args);
        let inner_block = parse_quote!(
            (#init_expr).run(|#var_name|{
                #(#inner_output)*
            }).expect(concat!("Could not upgrade ", stringify!(#var_name)));
        );
        output.push(inner_block);
    }
    output
}
