use crate::middle::ir::IR;

pub struct AssetOptimizer {
    pub minify: bool,
    pub remove_comments: bool,
}

impl AssetOptimizer {
    fn minify_js(&self, source: &str) -> String {
        use std::fs;
        use std::process::Command;

        let input = std::env::temp_dir().join("loi_temp.js");
        let output = std::env::temp_dir().join("loi_temp.min.js");

        fs::write(&input, source).unwrap();

        Command::new("esbuild")
            .arg(&input)
            .arg("--minify")
            .arg(format!("--outfile={}", output.display()))
            .status()
            .unwrap();

        fs::read_to_string(output).unwrap()
    }
    pub fn optimize(&self, ir: IR, ext: &str) -> IR {
        match ir {
            IR::Raw(content) => {
                let mut optimized = content;
                if self.remove_comments {
                    optimized = self.strip_comments(&optimized, ext);
                }
                if self.minify && matches!(ext, "js" | "ts") {
                    optimized = self.minify_js(&optimized);
                }
                IR::Raw(optimized)
            }
            // Return complex IR as-is
            ir => ir,
        }
    }
    fn strip_comments(&self, content: &str, lang: &str) -> String {
        let pattern = match lang {
            "js" | "ts" | "css" => r"(?s)(//.*?\n|/\*.*?\*/)",
            _ => return content.to_string(),
        };
        // Note: In production, compile the Regex once and store it in the struct
        regex::Regex::new(pattern)
            .map(|re| re.replace_all(content, "").to_string())
            .unwrap_or_else(|_| content.to_string())
    }
}
