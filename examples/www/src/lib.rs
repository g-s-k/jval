use std::ops::Range;

use jval::Json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Error {
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
pub fn validate(json: &str) -> Option<Error> {
    json.parse::<Json>()
        .map_err(|(_, Range { start, end })| Error { start, end })
        .err()
}
