use crate::registry::registry::Registry;

// src/registry/display.rs
pub trait RegistryDisplay {
    fn render_table(&self);
}

impl RegistryDisplay for Registry {
    fn render_table(&self) {
        println!("{:-<60}", "");
        println!(
            "{:<20} | {:<10} | {:<10} | {:<10}",
            "Namespace", "Name", "Version", "Cap"
        );
        println!("{:-<60}", "");

        for file in &self.files {
            println!(
                "{:<20} | {:<10} | {:<10} | {:<10}",
                format!("/{}", file.namespace),
                file.name,
                file.version,
                file.utter.as_deref().unwrap_or("-")
            );
        }
        println!("{:-<60}", "");
    }
}
