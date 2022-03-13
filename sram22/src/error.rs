use thiserror::Error;

#[derive(Debug, Error)]
pub enum Sram22Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("magic error: {0}")]
    Magic(#[from] magic_vlsi::error::MagicError),

    #[error("invalid template: {0}")]
    Template(#[from] handlebars::TemplateError),

    #[error("error rendering template: {0}")]
    RenderTemplate(#[from] handlebars::RenderError),

    #[error("error running LVS: {0}")]
    Lvs(String),

    #[error("error serializing/deserializing JSON: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Sram22Error>;
