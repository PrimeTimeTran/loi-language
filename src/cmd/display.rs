use tabled::Tabled;

#[derive(Tabled)]
pub struct FileView {
    pub namespace: String,
    pub name: String,
    pub capability: String,
}

pub trait RegistryPrinter {
    fn render_list(&self);
}
