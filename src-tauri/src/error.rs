use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("File access error: {message}")]
    IoError { message: String },

    #[error("XML parse error: {file} - {message}")]
    XmlParseError { file: String, message: String },

    #[error("Backup error: {message}")]
    BackupError { message: String },

    #[error("Savegame not found: {path}")]
    SavegameNotFound { path: String },

    #[error("Image processing error: {message}")]
    ImageError { message: String },
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        match self {
            AppError::IoError { message } => {
                state.serialize_field("code", "errors.ioError")?;
                state.serialize_field(
                    "params",
                    &std::collections::HashMap::from([("message", message.as_str())]),
                )?;
            }
            AppError::XmlParseError { file, message } => {
                state.serialize_field("code", "errors.xmlParseError")?;
                state.serialize_field(
                    "params",
                    &std::collections::HashMap::from([
                        ("file", file.as_str()),
                        ("message", message.as_str()),
                    ]),
                )?;
            }
            AppError::BackupError { message } => {
                state.serialize_field("code", "errors.backupError")?;
                state.serialize_field(
                    "params",
                    &std::collections::HashMap::from([("message", message.as_str())]),
                )?;
            }
            AppError::SavegameNotFound { path } => {
                state.serialize_field("code", "errors.savegameNotFound")?;
                state.serialize_field(
                    "params",
                    &std::collections::HashMap::from([("path", path.as_str())]),
                )?;
            }
            AppError::ImageError { message } => {
                state.serialize_field("code", "errors.imageError")?;
                state.serialize_field(
                    "params",
                    &std::collections::HashMap::from([("message", message.as_str())]),
                )?;
            }
        }
        state.end()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        AppError::ImageError {
            message: err.to_string(),
        }
    }
}

impl From<quick_xml::DeError> for AppError {
    fn from(err: quick_xml::DeError) -> Self {
        AppError::XmlParseError {
            file: String::new(),
            message: err.to_string(),
        }
    }
}
