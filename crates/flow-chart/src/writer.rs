use crate::{config::Config, render::RenderedFile, ui::render_output};

pub trait OutputWriter {
    fn write_file(&self, files: Vec<RenderedFile>, config: &Config) -> String;
}

pub struct MarkdownWriter;

impl OutputWriter for MarkdownWriter {
    fn write_file(&self, files: Vec<RenderedFile>, config: &Config) -> String {
        let mut output = String::new();
        let mut empty = vec![];

        let mut sorted_files = files;
        sorted_files.sort_by(|a, b| a.path.cmp(&b.path));

        for f in sorted_files {
            let relative_path = f
                .path
                .strip_prefix(&config.analysis_root)
                .unwrap_or(&f.path);

            if f.is_empty {
                empty.push(format!("  {}", relative_path.display()));
                continue;
            }
            let depth = relative_path.components().count();
            let level = depth.clamp(1, 6);
            let header_prefix = "#".repeat(level);

            output.push_str(&format!(
                "{} {}\n\n",
                header_prefix,
                relative_path.display()
            ));

            if config.format.wrap_in_code_blocks {
                let ext = relative_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt");
                output.push_str(&format!("```{}\n{}\n```\n\n", ext, f.body));
            } else {
                output.push_str(&format!("{}\n\n", f.body));
            }
        }

        if !empty.is_empty() {
            output.push_str("\n\n# EMPTY FILES\n");
            output.push_str(&empty.join("\n"));
        }

        render_output(&output, config)
    }
}
