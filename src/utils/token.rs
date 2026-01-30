use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashSet;
use tiktoken_rs::{cl100k_base, CoreBPE};
use tracing::error;

#[derive(Debug, Default)]
pub struct TokenCounts {
    pub total: i32,
    pub spec: i32,
    pub wb: i32,
    pub other: i32,
}

// Global cached tokenizer (Thread-safe, initialized once)
static BPE: Lazy<Result<CoreBPE, anyhow::Error>> = Lazy::new(|| {
    cl100k_base().map_err(|e| anyhow::anyhow!("Failed to load cl100k_base tokenizer: {}", e))
});

pub fn calculate_card_tokens(json: &Value) -> TokenCounts {
    let mut counts = TokenCounts::default();

    // Use the global cached BPE tokenizer
    let bpe = match BPE.as_ref() {
        Ok(b) => b,
        Err(e) => {
            error!("Tokenizer initialization failed: {}", e);
            return counts;
        }
    };

    // 1. Spec Tokens
    let spec_fields = [
        "description",
        "personality",
        "scenario",
        "first_mes",
        "mes_example",
        "system_prompt",
        "post_history_instructions",
        "alternate_greetings",
        "talkativeness",
    ];

    let mut spec_tokens = 0;

    // Helper to traverse and collect unique values for spec fields
    // We look at root and root.data
    let mut spec_values: HashSet<String> = HashSet::new();

    fn collect_field_value(json: &Value, field: &str, set: &mut HashSet<String>) {
        if let Some(val) = json.get(field) {
            match val {
                Value::String(s) => {
                    if !s.is_empty() {
                        set.insert(s.clone());
                    }
                }
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::String(s) = item {
                            if !s.is_empty() {
                                set.insert(s.clone());
                            }
                        }
                    }
                }
                Value::Number(n) => {
                    set.insert(n.to_string());
                }
                Value::Bool(b) => {
                    set.insert(b.to_string());
                }
                _ => {}
            }
        }
    }

    for field in spec_fields {
        collect_field_value(json, field, &mut spec_values);
        if let Some(data) = json.get("data") {
            collect_field_value(data, field, &mut spec_values);
        }
    }

    // Special handling for creator_notes / creatorcomment aliasing
    // (Already handled by including both in loop and using set to dedup if values are identical)

    for s in &spec_values {
        // Encode
        spec_tokens += bpe.encode_with_special_tokens(s).len();
    }
    counts.spec = spec_tokens as i32;

    // 2. World Book Tokens
    let mut wb_tokens = 0;
    let mut wb_values: HashSet<String> = HashSet::new();

    // Look for character_book in root and data
    // It's usually a separate object or embedded.
    // Spec says "character_book" field.

    fn collect_wb_recursive(
        val: &Value,
        set: &mut HashSet<String>,
        bpe: &tiktoken_rs::CoreBPE,
    ) -> usize {
        let mut count = 0;
        match val {
            Value::String(s) => {
                if !s.is_empty() && set.insert(s.clone()) {
                    // Insert returns true if new
                    count += bpe.encode_with_special_tokens(s).len();
                }
            }
            Value::Object(map) => {
                for (_, v) in map {
                    count += collect_wb_recursive(v, set, bpe);
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    count += collect_wb_recursive(v, set, bpe);
                }
            }
            _ => {}
        }
        count
    }

    if let Some(cb) = json.get("character_book") {
        wb_tokens += collect_wb_recursive(cb, &mut wb_values, bpe);
    }
    if let Some(data) = json.get("data") {
        if let Some(cb) = data.get("character_book") {
            wb_tokens += collect_wb_recursive(cb, &mut wb_values, bpe);
        }
    }
    counts.wb = wb_tokens as i32;

    // 3. Total Tokens
    // "Duplicates not counted". We will traverse the entire JSON, collect ALL unique string/primitive values, and sum them.
    let mut total_values: HashSet<String> = HashSet::new();

    fn collect_all_recursive(
        val: &Value,
        set: &mut HashSet<String>,
        bpe: &tiktoken_rs::CoreBPE,
    ) -> usize {
        let mut count = 0;
        match val {
            Value::String(s) => {
                // If this string was already counted in Spec or WB check?
                // No, just global unique set for Total.
                if !s.is_empty() && set.insert(s.clone()) {
                    count += bpe.encode_with_special_tokens(s).len();
                }
            }
            Value::Number(n) => {
                let s = n.to_string();
                if set.insert(s.clone()) {
                    count += bpe.encode_with_special_tokens(&s).len();
                }
            }
            Value::Bool(b) => {
                let s = b.to_string();
                if set.insert(s.clone()) {
                    count += bpe.encode_with_special_tokens(&s).len();
                }
            }
            Value::Object(map) => {
                for (k, v) in map {
                    if k == "regex_scripts" {
                        continue;
                    }
                    count += collect_all_recursive(v, set, bpe);
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    count += collect_all_recursive(v, set, bpe);
                }
            }
            _ => {}
        }
        count
    }

    counts.total = collect_all_recursive(json, &mut total_values, bpe) as i32;

    counts.other = std::cmp::max(0, counts.total - counts.spec - counts.wb);

    counts
}
