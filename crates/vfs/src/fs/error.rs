#[derive(Debug)]
pub enum FSError {
    NotFound,
    PermissionDenied,
    IoError,
    AlreadyExists,
    InvalidPath,
}
