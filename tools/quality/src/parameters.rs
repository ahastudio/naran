//! The parameters of a function: how many, and whether they read well at the
//! place of the call.

use quote::ToTokens;

use crate::{Finding, Place};

/// The most parameters a function may take.
pub const LIMIT: usize = 3;

/// What GPUI passes to every handler; it does not count as a parameter.
const CONTEXTS: [&str; 5] = ["Window", "Context", "App", "AsyncApp", "AsyncWindowContext"];

/// Number and text types that are easy to pass in the wrong order.
const PLAIN: [&str; 16] = [
    "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32",
    "f64", "str", "String",
];

/// Neighbours whose order is their meaning.
const ORDERED: [[&str; 2]; 8] = [
    ["x", "y"],
    ["width", "height"],
    ["row", "column"],
    ["line", "column"],
    ["start", "end"],
    ["from", "to"],
    ["old", "new"],
    ["key", "value"],
];

/// What is wrong with the parameters of the function at `place`.
pub fn check(place: &Place, signature: &syn::Signature) -> Vec<Finding> {
    let typed: Vec<Typed> = signature.inputs.iter().filter_map(Typed::of).collect();
    let flags = typed.iter().filter_map(Typed::flag);
    let pairs = typed
        .iter()
        .zip(typed.iter().skip(1))
        .filter_map(neighbours);
    let found = too_many(typed.len()).into_iter().chain(flags).chain(pairs);
    found.map(|message| place.message(&message)).collect()
}

fn too_many(count: usize) -> Option<String> {
    (count > LIMIT).then(|| format!("has {count} parameters"))
}

/// A parameter other than `self` and the GPUI contexts.
struct Typed {
    name: String,
    kind: String,
    plain: String,
}

impl Typed {
    fn of(input: &syn::FnArg) -> Option<Self> {
        let syn::FnArg::Typed(typed) = input else {
            return None;
        };
        let plain = plain_name(&typed.ty).unwrap_or_default();
        (!CONTEXTS.contains(&plain.as_str())).then(|| Self {
            name: typed.pat.to_token_stream().to_string(),
            kind: typed.ty.to_token_stream().to_string(),
            plain,
        })
    }

    fn flag(&self) -> Option<String> {
        let message = format!("takes the flag `{}`", self.name);
        (self.plain == "bool").then_some(message)
    }

    fn is_plain(&self) -> bool {
        PLAIN.contains(&self.plain.as_str())
    }
}

/// The last name of a type, behind any references: `Context` for
/// `&mut Context<Self>`.
fn plain_name(kind: &syn::Type) -> Option<String> {
    match kind {
        syn::Type::Reference(reference) => plain_name(&reference.elem),
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn neighbours((first, second): (&Typed, &Typed)) -> Option<String> {
    let mixable = first.is_plain() && first.kind == second.kind;
    let names = [first.name.as_str(), second.name.as_str()];
    let [one, other] = names;
    let message = format!("takes `{one}` and `{other}` of one type in a row");
    (mixable && !ORDERED.contains(&names)).then_some(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(function: &str) -> Vec<String> {
        let function: syn::ItemFn = syn::parse_str(function).unwrap();
        let place = Place::of(std::path::Path::new("a.rs"), &function.sig.ident);
        let found = check(&place, &function.sig).into_iter();
        found.map(|finding| finding.message).collect()
    }

    #[test]
    fn more_than_three_parameters_are_too_many_but_self_and_contexts_do_not_count() {
        assert_eq!(
            messages("fn f(a: A, b: B, c: C, d: D) {}"),
            ["f has 4 parameters"]
        );
        let handler =
            "fn f(&mut self, a: &A, b: B, c: C, window: &mut Window, cx: &mut Context<Self>) {}";
        assert_eq!(messages(handler), Vec::<String>::new());
        let contexts = "fn f(cx: &mut App, async_cx: &mut AsyncApp, w: &mut AsyncWindowContext) {}";
        assert_eq!(messages(contexts), Vec::<String>::new());
    }

    #[test]
    fn a_bool_parameter_is_a_flag() {
        assert_eq!(
            messages("fn f(staged: bool) {}"),
            ["f takes the flag `staged`"]
        );
        assert_eq!(
            messages("fn f(item: (bool, A), list: &[bool], pick: fn()) {}"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn neighbours_of_one_plain_type_are_found_unless_their_order_is_their_meaning() {
        assert_eq!(
            messages("fn f(skip: usize, limit: usize) {}"),
            ["f takes `skip` and `limit` of one type in a row"]
        );
        assert_eq!(
            messages("fn f(a: &str, b: &str) {}"),
            ["f takes `a` and `b` of one type in a row"]
        );
        assert_eq!(
            messages("fn f(a: usize, b: u32, c: Path) {}"),
            Vec::<String>::new()
        );
        assert_eq!(messages("fn f(x: f32, y: f32) {}"), Vec::<String>::new());
        assert_eq!(
            messages("fn f(row: usize, column: usize) {}"),
            Vec::<String>::new()
        );
        assert_eq!(
            messages("fn f(from: &str, to: &str) {}"),
            Vec::<String>::new()
        );
    }
}
