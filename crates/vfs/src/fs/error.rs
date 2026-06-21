use core::fmt;

#[derive(Debug)]
pub enum FSError {
    NotFound,
    PermissionDenied,
    IoError,
    AlreadyExists,
    InvalidPath,
}

impl fmt::Display for FSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            FSError::NotFound => "file or directory not found",
            FSError::PermissionDenied => "permission denied",
            FSError::IoError => "I/O error",
            FSError::AlreadyExists => "already exists",
            FSError::InvalidPath => "invalid path",
        };

        write!(f, "{msg}")
    }
}

impl std::error::Error for FSError {}
