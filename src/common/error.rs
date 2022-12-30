pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed IO operation: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Failed db statement: {0}")]
    DBError(#[from] rusqlite::Error),

    #[error("Failed web request: {0}")]
    WebError(#[from] reqwest::Error),

    #[error("Failed to parse html for {0}, maybe the website layout changed?")]
    HtmlParseError(String),

    #[error("Unknown availability enum: {0}")]
    AvailabilityEnumError(String),

    #[error("Error sending discord notification: {0}")]
    DiscordError(String)
}