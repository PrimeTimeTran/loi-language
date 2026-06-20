use syn::visit::{self, Visit};

use crate::{
    config::Config,
    language::SymbolRegistry,
    ui::{render_enum, render_struct},
};

pub struct MyAnalyzer<'a> {
    pub config: &'a Config,
    pub items: &'a [syn::Item],
    pub rendered_output: Vec<String>,
    pub registry: SymbolRegistry,
}

impl<'a> Visit<'a> for MyAnalyzer<'a> {
    fn visit_item_struct(&mut self, i: &'a syn::ItemStruct) {
        let rendered = render_struct(i, self.config, "".to_string(), self.items);
        self.registry.structs.push(rendered);
        visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'a syn::ItemEnum) {
        let rendered = render_enum(i, self.config, "".to_string());
        self.registry.enums.push(rendered);
        visit::visit_item_enum(self, i);
    }
}
