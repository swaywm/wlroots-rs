extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{ItemFn, Stmt, UseTree, ItemUse, Item,
          parse::{self, Parse, ParseStream},
          punctuated::Punctuated};

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
    // TODO Pop and then report error if not all used
    fn is_handle(&self, name: Ident) -> bool {
        self.vars.contains(&name)
    }
}

/// Attribute to automatically call the `run` method on handles with the
/// remaining block of code.
///
/// The name of the variable you want to use as the upgraded handle should be
/// provided as an argument to the attribute. It does not need to be the same
/// as the handle variable.
///
/// The syntax in the code should be `use $handle as $upgraded_handle`.
/// E.g the variable in the code that stores the handle should go on the
/// **left** and the variable you used in the attribute declaration should
/// go on the **right**.
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
///         use compositor as compositor;
///         let keyboard = keyboard;
///         use compositor as compositor;
///         use keyboard as keyboard;
///         let server: &mut ::Server = compositor.into();
///         server.keyboards.push(keyboard.weak_reference());
///         // Now that we have at least one keyboard, update the seat capabilities.
///         let seat = &server.seat.seat;
///         use seat as seat;
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
        use {Stmt::Item, Item::Use, UseTree::Rename};
        match stmt.clone() {
            Item(Use(ItemUse { tree: Rename(use_stmt), ..})) => {
                if args.is_handle(use_stmt.ident.clone()) {
                    inner = Some((use_stmt.ident, use_stmt.rename));
                    break
                }
                output.push(stmt)
            },
            _ => output.push(stmt)
        }
    }
    if let Some((handle, dehandle)) = inner {
        let inner_output = build_block(input, args);
        let inner_block = parse_quote!(
            (#handle).run(|#dehandle|{
                #(#inner_output)*
            }).expect(concat!("Could not upgrade handle ",
                              stringify!(#handle), " to ",
                              stringify!(#dehandle)));
        );
        output.push(inner_block);
    }
    output
}
