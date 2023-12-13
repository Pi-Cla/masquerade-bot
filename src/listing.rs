use std::{collections::HashMap, fmt::Write};

use volty::{
    http::routes::channels::message_send::SendableMessage,
    types::channels::message::{Interactions, Message},
};

use crate::{models::Profile, Bot, Error};

pub const PER_PAGE: usize = 5;

pub fn get_page(profiles: &[Profile], page: usize) -> String {
    let last_page = (profiles.len().max(1) - 1) / PER_PAGE;
    let mut text = format!(
        "[](T:L)[](P:{page}){}/{}\n| Name | Display Name | Avatar | Colour |\n|-|-|-|-|",
        page + 1,
        last_page + 1
    );

    let start = page * PER_PAGE;
    let end = (start + PER_PAGE).min(profiles.len());
    if start >= profiles.len() {
        return text;
    }

    for p in &profiles[start..end] {
        write!(
            &mut text,
            "\n|{}|{}|{}|{}|",
            p.name,
            p.display_name.as_deref().unwrap_or(""),
            p.avatar
                .as_ref()
                .map(|u| format!("[Link](<{u}>)"))
                .unwrap_or_default(),
            p.colour.as_deref().unwrap_or("")
        )
        .unwrap();
    }

    text
}

impl Bot {
    pub async fn list_profiles(&self, message: &Message) -> Result<(), Error> {
        let profiles = self
            .db
            .get_profiles(&message.author_id)
            .await
            .unwrap_or_default();
        let page = get_page(&profiles, 0);
        let send = SendableMessage::new()
            .content(page)
            .interactions(Interactions::new(["👈", "👉"]).restrict())
            .reply(message.id.clone());
        self.http.send_message(&message.channel_id, send).await?;
        Ok(())
    }

    pub async fn on_listing_react(
        &self,
        message: &Message,
        reply: &Message,
        data: HashMap<&str, &str>,
        emoji_id: &str,
    ) -> Result<(), Error> {
        let profiles = self
            .db
            .get_profiles(&reply.author_id)
            .await
            .unwrap_or_default();
        let last_page = (profiles.len().max(1) - 1) / PER_PAGE;
        let current_page: usize = data
            .get("P")
            .copied()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);
        let page = match emoji_id {
            "👈" => {
                if current_page == 0 {
                    last_page
                } else {
                    current_page - 1
                }
            }
            "👉" => {
                if current_page >= last_page {
                    0
                } else {
                    current_page + 1
                }
            }
            _ => unreachable!(),
        };
        let page = get_page(&profiles, page);
        if Some(&page) == message.content.as_ref() {
            return Ok(());
        }
        self.http
            .edit_message(&message.channel_id, &message.id, page)
            .await?;
        Ok(())
    }
}
