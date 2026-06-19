use colored::Colorize;
use owo_colors::{OwoColorize, colors::css::*};
use strum::IntoEnumIterator;
use tabled::{
    Table, Tabled,
    settings::{Color, Modify, Style, object::Rows},
};

use crate::{
    cli::command::Command,
    registry::file_meta::FileMeta,
    registry::prog_registry::{FileStack, Registry},
};

#[derive(Tabled)]
pub struct FileView {
    pub namespace: String,
    #[tabled(rename = "#")]
    pub index: usize,
    #[tabled(rename = "FS Name")]
    pub filename: String,
    #[tabled(rename = "Active")]
    pub active: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Utter")]
    pub utter: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Tag")]
    pub tag: String,
    #[tabled(rename = "Ext")]
    pub ext: String,
    #[tabled(rename = "Capabilities")]
    pub capabilities: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListFilter {
    #[default]
    Active,
    Archived,
    All,
}

pub trait RegistryUI {
    fn render_header(&self, registry: &Registry);
    fn render_shortcuts(&self);
    fn render_list(&self, registry: &Registry, filter: ListFilter);
    fn render_tree(&self, registry: &Registry);
    fn render_version_history(&self, registry: &Registry, target: Option<&str>);
    fn render_capability_map(&self, registry: &Registry);
    fn render_diff(&self, registry: &Registry, a: &str, b: &str);
    fn render_file_contents(&self, path: &std::path::Path, contents: &str);
    fn render_error(&self, text: String);
}

pub struct RegistryRenderer;

impl RegistryUI for RegistryRenderer {
    fn render_file_contents(&self, path: &std::path::Path, contents: &str) {
        println!(
            "\n{}",
            format!("----- Viewing: {} ", path.display())
                .bold()
                .fg::<LightCoral>()
        );
        println!("{}", contents);
        println!(
            "{}",
            "-----------------------------------".fg::<LightCoral>()
        );
    }
    fn render_header(&self, registry: &Registry) {
        let total_files = registry.files.len() + registry.files_archive.len();
        let files = registry.files.values().filter(|f| f.active).count();

        let total_utters = registry
            .files
            .values()
            .filter_map(|f| f.utter.as_ref())
            .count();

        let total_versions = registry.files.values().filter(|f| f.version > 0).count();
        println!("\n{}", Theme::header("--- Metrics ---"));
        println!(
            "files: {:<3} active: {:<3} versions: {:<3} utters:{:<3}",
            total_files, files, total_versions, total_utters
        );
    }
    fn render_shortcuts(&self) {
        let mut cmds: Vec<_> = Command::iter()
            .map(|c| c.metadata())
            .filter(|m| !m.hidden)
            .collect();

        cmds.sort_by(|a, b| b.weight.cmp(&a.weight));

        print!("\n{}\n", Theme::header("--- Shortcuts ---"));

        for cmd in cmds {
            match cmd.alias {
                Some(a) => {
                    print!("{} ({})  ", cmd.label, a.dimmed());
                }
                None => {
                    print!("{}  ", cmd.label.green());
                }
            }
        }

        println!("\n");
    }
    fn render_list(&self, registry: &Registry, filter: ListFilter) {
        let mut rows: Vec<FileView> = Vec::new();
        let mut flat: Vec<(&FileMeta, bool)> = Vec::new();

        let mut stacks: Vec<&FileStack> = registry.stacks.iter().collect();

        stacks.sort_by(|a, b| {
            b.active_file
                .version
                .cmp(&a.active_file.version)
                .then_with(|| b.active_file.name.cmp(&a.active_file.name))
        });
        for stack in &mut stacks {
            let mut sorted_archive: Vec<&FileMeta> = stack
                .files
                .iter()
                .filter(|f| f.id != stack.active_file.id) // <--- ADD THIS FILTER
                .collect();

            sorted_archive.sort_by(|a, b| a.version.cmp(&b.version));

            let group = sorted_archive
                .into_iter()
                .chain(std::iter::once(&stack.active_file));

            for f in group {
                let is_active = f.id == stack.active_file.id;

                let should_show = match filter {
                    ListFilter::Active => is_active,
                    ListFilter::Archived => !is_active,
                    ListFilter::All => true,
                };

                if should_show {
                    flat.push((f, is_active));
                }
            }
        }
        for (i, (f, is_active)) in flat.iter().enumerate() {
            rows.push(FileView {
                index: i + 1,
                filename: f.filename.clone(),
                namespace: f.namespace.clone(),
                active: is_active.to_string(),
                name: f.name.clone(),
                utter: f.utter.clone().unwrap_or_default(),
                ext: f.ext.clone(),
                version: f.version.to_string(),
                tag: f.tag.clone().unwrap_or_default(),
                capabilities: f.capabilities.concat(),
            });
        }

        let mut table = Table::new(rows);
        table.with(Style::modern());

        // ---------------------------
        // 3. DIMMING (aligned with flat)
        // ---------------------------
        for (i, (_, is_active)) in flat.iter().enumerate() {
            if !is_active {
                table.with(Modify::new(Rows::new(i + 1..i + 2)).with(Color::FG_BRIGHT_BLACK));
            }
        }

        println!("\n{}", Theme::header("--- Registry Status ---"));
        println!("{}\n", table);
    }
    fn render_tree(&self, registry: &Registry) {
        use std::collections::BTreeMap;

        println!("\n{}", Theme::header("--- Namespace Tree ---"));

        // let mut map: BTreeMap<String, Vec<&FileMeta>> = BTreeMap::new();
        let mut map: BTreeMap<String, Vec<&FileMeta>> = BTreeMap::new();

        // 1. Before
        // for f in &registry.files {
        //     map.entry(f.namespace.clone()).or_default().push(f);
        // }

        // 2. Now
        for f in registry.files.values() {
            map.entry(f.namespace.clone()).or_default().push(f);
        }

        // 3. Optional
        // for (_, f) in &registry.files {
        //     println!("ID: {} | Name: {}", id, file.name);
        // }

        for (ns, files) in map {
            println!("\n{}", ns.bold().cyan());

            for f in files {
                let mark = if f.active {
                    "●".green()
                } else {
                    "○".dimmed()
                };
                println!(
                    "  {} {} {}",
                    mark,
                    f.name,
                    f.utter.as_deref().unwrap_or("").dimmed()
                );
            }
        }

        println!();
    }

    fn render_version_history(&self, registry: &Registry, target: Option<&str>) {
        println!("\n{}", Theme::header("--- Version Audit Trail ---"));

        let Some(target) = target else {
            println!("{}", "Usage: history <name>".yellow());
            return;
        };

        let mut versions: Vec<&FileMeta> = registry
            .files
            .values()
            .filter(|f| f.name == target)
            .collect();

        if versions.is_empty() {
            println!("{}", "No history found".red());
            return;
        }

        versions.sort_by_key(|f| f.version);

        println!("\n{}:", target.bold().green());

        for v in versions {
            println!(
                "  v{} {} {}",
                v.version.to_string().cyan(),
                if v.active {
                    "active".green()
                } else {
                    "archived".dimmed()
                },
                v.utter.as_deref().unwrap_or("").dimmed()
            );
        }

        println!();
    }

    fn render_capability_map(&self, registry: &Registry) {
        use std::collections::BTreeMap;

        println!("\n{}", Theme::header("--- Capability Matrix ---"));

        let mut map: BTreeMap<String, Vec<&FileMeta>> = BTreeMap::new();

        for f in registry.files.values() {
            for cap in &f.capabilities {
                map.entry(cap.clone()).or_default().push(f);
            }
        }

        for (cap, files) in map {
            println!("\n{}", cap.bold().cyan());

            for f in files {
                let status = if f.active { "●" } else { "○" }.green();
                println!("  {} {}", status, f.name);
            }
        }

        println!();
    }

    fn render_diff(&self, registry: &Registry, a: &str, b: &str) {
        println!("\n{}", Theme::header("--- Semantic Diff ---"));

        let a = registry.files.values().find(|f| f.name == a);
        let b = registry.files.values().find(|f| f.name == b);

        match (a, b) {
            (Some(a), Some(b)) => {
                println!("\n{} vs {}\n", a.name.bold(), b.name.bold());

                fn line(label: &str, x: &str, y: &str) {
                    let diff = if x == y { "" } else { "●" };
                    println!(
                        "{:<12} {:<20} {:<20} {}",
                        label,
                        x.dimmed(),
                        y.dimmed(),
                        diff.red()
                    );
                }

                line("name", &a.name, &b.name);
                line("version", &a.version.to_string(), &b.version.to_string());
                line(
                    "tag",
                    a.tag.as_deref().unwrap_or(""),
                    b.tag.as_deref().unwrap_or(""),
                );
                line("ext", &a.ext, &b.ext);
                line(
                    "utter",
                    a.utter.as_deref().unwrap_or(""),
                    b.utter.as_deref().unwrap_or(""),
                );
            }

            _ => {
                println!("{}", "Files not found".red());
            }
        }

        println!();
    }
    fn render_error(&self, text: String) {
        println!("Error: {}", text);
    }
}

pub struct Theme;

impl Theme {
    pub fn light_grey(text: &str) -> String {
        format!("{}", text.fg::<DarkGray>())
    }
    pub fn header(text: &str) -> String {
        Self::light_grey(text)
    }
    pub fn error(text: &str) -> String {
        format!("{}", text.red())
    }
    pub fn success(text: &str) -> String {
        format!("{}", text.green())
    }
    pub fn highlight(text: &str) -> String {
        format!("{}", text.yellow())
    }
}
