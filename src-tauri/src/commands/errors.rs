#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Failed to watch directory: {0}")]
    WatchError(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Recording is only supported on Windows")]
    UnsupportedPlatform,
    #[error("Failed to initialize recorder: {0}")]
    InitializationError(String),
    #[error("Dolphin window not found")]
    WindowNotFound,
    #[error("Recording failed: {0}")]
    RecordingFailed(String),
}

#[derive(serde::Serialize)]
#[serde(tag = "name", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorName {
    Io(String),
    FromUtf8Error(String),
    WatchError(String),
    InvalidPath(String),
    UnsupportedPlatform(String),
    InitializationError(String),
    WindowNotFound(String),
    RecordingFailed(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let message = self.to_string();
        let name = match self {
            Self::Io(_) => ErrorName::Io(message),
            Self::Utf8(_) => ErrorName::FromUtf8Error(message),
            Self::WatchError(_) => ErrorName::WatchError(message),
            Self::InvalidPath(_) => ErrorName::InvalidPath(message),
            Self::UnsupportedPlatform => ErrorName::UnsupportedPlatform(message),
            Self::InitializationError(_) => ErrorName::InitializationError(message),
            Self::WindowNotFound => ErrorName::WindowNotFound(message),
            Self::RecordingFailed(_) => ErrorName::RecordingFailed(message),
        };
        name.serialize(serializer)
    }
}
