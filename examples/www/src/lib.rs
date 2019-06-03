use std::ops::Range;

use jval::{Json, Spacing};
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

#[wasm_bindgen]
pub fn format_packed(json: &str) -> Option<String> {
    json.parse::<Json>()
        .map(|j: Json| {
            let mut s = String::new();
            j.print(&Spacing::None, &mut s).unwrap();
            s
        })
        .ok()
}

#[wasm_bindgen]
pub fn format_tabs(json: &str) -> Option<String> {
    json.parse::<Json>()
        .map(|j: Json| {
            let mut s = String::new();
            j.print(&Spacing::Tab, &mut s).unwrap();
            s
        })
        .ok()
}

#[wasm_bindgen]
pub fn format_spaces(json: &str, spacing: usize) -> Option<String> {
    json.parse::<Json>()
        .map(|j: Json| {
            let mut s = String::new();
            j.print(&Spacing::Space(spacing), &mut s).unwrap();
            s
        })
        .ok()
}
