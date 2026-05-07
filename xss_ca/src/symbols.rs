//! Symbol table and dynamic value generators.

use crate::GROUP;
use core::iter::repeat_with;
use rand::{RngExt as _, distr::Alphanumeric, rng};
use std::collections::HashMap;

/// Generates a random valid URL for use as a src attribute value.
///
/// Produces `src="https://DOMAIN.example.com/PATH"` where domain and path are
/// random lowercase alphanumeric strings. The loop retries until the URL
/// contains no characters that would break an HTML attribute value.
pub fn generate_src_url() -> String {
    let mut rng = rng();
    loop {
        let domain_len = rng.random_range(4..=10);
        let path_len = rng.random_range(3..=8);
        let domain =
            repeat_with(|| char::from(rng.sample(Alphanumeric))).take(domain_len).collect::<String>().to_lowercase();
        let path =
            repeat_with(|| char::from(rng.sample(Alphanumeric))).take(path_len).collect::<String>().to_lowercase();
        let url = format!("src=\"https://{domain}.example.com/{path}\"");
        if !url.contains('\'') && !url.contains('>') && !url.contains(' ') {
            return url;
        }
    }
}

/// Generates a random sequence of 1–4 space or tab characters.
pub fn generate_random_whitespace() -> String {
    let mut rng = rng();
    let count = rng.random_range(1..=4);
    repeat_with(|| if rng.random_bool(0.5_f64) { ' ' } else { '\t' }).take(count).collect()
}

/// Builds the symbol lookup table used by all translation functions.
///
/// Each key is a logical token name and each value is the concrete string it
/// expands to. Encoding tokens are stored as empty strings because their
/// expansion is performed at translation time by
/// [`crate::encoding::apply_encoding`].
pub fn symbol_map() -> HashMap<&'static str, String> {
    let mut map: HashMap<&'static str, String> = HashMap::new();

    // Script tags
    map.insert("script_open", String::from("<script>"));
    map.insert("Script_open", String::from("<Script>"));
    map.insert("SCRIPT_open", String::from("<SCRIPT>"));
    map.insert("script_close", String::from("</script>"));
    map.insert("Script_close", String::from("</Script>"));
    map.insert("SCRIPT_close", String::from("</SCRIPT>"));

    // Payloads
    map.insert("squote_payload", format!("console.log('{GROUP}')"));
    map.insert("dquote_payload", format!("console.log(\"{GROUP}\")"));
    map.insert("eval_payload", format!("eval('console.log(\"{GROUP}\")')"));
    map.insert("backtick_payload", format!("console.log(`{GROUP}`)"));

    // Whitespace variants — space is generated dynamically
    map.insert("none", String::new());
    map.insert("space", generate_random_whitespace());
    map.insert("tab", String::from("\t"));
    map.insert("newline", String::from("\n"));

    // Encoding — resolved at translate time
    map.insert("url_encoded", String::new());
    map.insert("html_entity", String::new());
    map.insert("unicode_escape", String::new());

    // HTML tag names
    map.insert("img", String::from("img"));
    map.insert("svg", String::from("svg"));
    map.insert("body", String::from("body"));
    map.insert("input", String::from("input"));
    map.insert("video", String::from("video"));
    map.insert("script", String::from("script"));

    // src attribute variants — src_valid is generated dynamically
    map.insert("empty", String::new());
    map.insert("src_x", String::from("src=\"x\""));
    map.insert("src_valid", generate_src_url());

    // DOM event handler names
    map.insert("onerror", String::from("onerror"));
    map.insert("onload", String::from("onload"));
    map.insert("onfocus", String::from("onfocus"));
    map.insert("onmouseover", String::from("onmouseover"));
    map.insert("onclick", String::from("onclick"));

    // Quote style tokens
    map.insert("dquote", String::from("\""));
    map.insert("squote", String::from("'"));
    map.insert("no_quote", String::new());

    // Tag-case tokens — handled procedurally in apply_case
    map.insert("lower", String::from("lower"));
    map.insert("upper", String::from("upper"));
    map.insert("mixed", String::from("mixed"));

    // Attribute-breakout sequences
    map.insert("dquote_close", String::from("\">"));
    map.insert("squote_close", String::from("'>"));
    map.insert("angle_close", String::from(">"));
    map.insert("slash", String::from("/"));
    map.insert("comment_close", String::from("-->"));
    map.insert("space_slash", String::from(" />"));

    // JavaScript context tokens
    map.insert("squote_break", String::from("'"));
    map.insert("dquote_break", String::from("\""));
    map.insert("backtick_break", String::from("`"));
    map.insert("bracket_break", String::from("]"));
    map.insert("semicolon", String::from(";"));
    map.insert("line_comment", String::from("//"));
    map.insert("block_comment", String::from("/*"));

    map
}
