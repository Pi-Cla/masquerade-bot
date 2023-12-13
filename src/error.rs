use volty::prelude::*;

#[derive(Debug)]
pub enum Error {
    BotMissing(Permission),
    UserMissing(Permission),

    Http(HttpError),
    Mongo(mongodb::error::Error),
    Validate(validator::ValidationErrors),
}

impl From<HttpError> for Error {
    fn from(value: HttpError) -> Self {
        Self::Http(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Mongo(value)
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(value: validator::ValidationErrors) -> Self {
        Self::Validate(value)
    }
}
