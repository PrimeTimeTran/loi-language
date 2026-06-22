pub fn init_logging() {
    tracing_subscriber::fmt::init();
}

pub fn success(message: &str) {
    println!("✓ {}", message);
}

pub fn error(message: &str) {
    eprintln!("✗ {}", message);
}
