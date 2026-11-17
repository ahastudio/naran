//! No body is written twice.

use std::collections::HashMap;

use quote::ToTokens;

use crate::{Finding, Place};

/// Bodies with fewer statements are too small to call a copy.
pub const SMALLEST: usize = 3;

/// The text of one function body, to compare with the others.
#[derive(Debug, Clone)]
pub struct Body {
    place: Place,
    text: String,
}

impl Body {
    /// None for a body too small to compare.
    pub fn of(place: &Place, body: &syn::Block) -> Option<Self> {
        let text = body.to_token_stream().to_string();
        let large = crate::statements(body) >= SMALLEST;
        large.then(|| Self {
            place: place.clone(),
            text,
        })
    }
}

/// A finding for every body that is the same as one before it.
pub fn check(bodies: &[Body]) -> Vec<Finding> {
    let mut first: HashMap<&str, &Place> = HashMap::new();
    let copies = bodies.iter().filter_map(|body| {
        let original = *first.entry(&body.text).or_insert(&body.place);
        copied(&body.place, original)
    });
    copies.collect()
}

fn copied(place: &Place, original: &Place) -> Option<Finding> {
    let at = format!("{}:{}", original.path.display(), original.line);
    let message = format!("has the same body as {} at {at}", original.name);
    (!std::ptr::eq(place, original)).then(|| place.message(&message))
}
