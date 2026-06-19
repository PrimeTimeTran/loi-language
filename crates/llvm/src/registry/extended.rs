pub enum WebTarget {
    Html,
    Css,
    Js,
}

pub const WEB_EXTENSIONS: &[&str] = &[
    "html", "css", "js", "json", "svg", "ts", "tsx", "jsx", "md", "graphql", "gql", "wasm",
];

pub fn web_target(ext: &str) -> Option<WebTarget> {
    match ext {
        "html" => Some(WebTarget::Html),
        "css" => Some(WebTarget::Css),
        "js" => Some(WebTarget::Js),
        _ => None,
    }
}

pub fn extract_web_target(name: &str) -> Option<WebTarget> {
    let base = name.strip_suffix(".loi")?;

    if base.ends_with(".html") {
        Some(WebTarget::Html)
    } else if base.ends_with(".css") {
        Some(WebTarget::Css)
    } else if base.ends_with(".js") {
        Some(WebTarget::Js)
    } else {
        None
    }
}
