struct Loi {
    pub name: String,
    pub description: String,
    pub category: String,
}

impl Loi {
    pub fn new(name: String, description: String, category: String) -> Self {
        Self {
            name,
            description,
            category,
        }
    }
}
