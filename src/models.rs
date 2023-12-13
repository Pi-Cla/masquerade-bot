use validator::Validate;
use volty::types::{
    channels::message::Masquerade,
    util::regex::{RE_COLOUR, RE_DISPLAY_NAME, RE_USERNAME},
};

#[derive(Clone, Debug)]
pub struct Author {
    pub message_id: String,
    pub user_id: String,
}

#[derive(Clone, Debug, Validate)]
pub struct Profile {
    pub user_id: String,
    #[validate(
        length(min = 1, message = "cannot be empty"),
        length(max = 32, message = "must be <= 32 characters"),
        regex(path = "RE_USERNAME", message = "contains invalid characters")
    )]
    pub name: String,
    #[validate(
        length(min = 1, message = "cannot be empty"),
        length(max = 32, message = "must be <= 32 characters"),
        regex(path = "RE_DISPLAY_NAME", message = "contains invalid characters")
    )]
    pub display_name: Option<String>,
    #[validate(
        length(min = 1, message = "cannot be empty"),
        length(max = 128, message = "must be <= 128 characters"),
        url(message = "isn't a valid url")
    )]
    pub avatar: Option<String>,
    #[validate(
        length(min = 1, message = "cannot be empty"),
        length(max = 128, message = "must be <= 128 characters"),
        regex(path = "RE_COLOUR", message = "not supported")
    )]
    pub colour: Option<String>,
}

impl Profile {
    pub fn new(user_id: &str, name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            name: name.to_string(),
            display_name: None,
            avatar: None,
            colour: None,
        }
    }
}

impl From<Profile> for Masquerade {
    fn from(val: Profile) -> Self {
        let name = val.display_name.unwrap_or(val.name);
        Self {
            name: Some(name),
            avatar: val.avatar,
            colour: val.colour,
        }
    }
}
