use std::collections::HashMap;

use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, to_document},
    options::{ClientOptions, UpdateOptions},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use validator::Validate;

use crate::{
    models::{Author, Profile},
    Error,
};

#[derive(Deserialize, Serialize)]
struct AuthorDoc {
    _id: String,
    user_id: String,
}

impl From<Author> for AuthorDoc {
    fn from(value: Author) -> Self {
        Self {
            _id: value.message_id,
            user_id: value.user_id,
        }
    }
}

impl From<AuthorDoc> for Author {
    fn from(value: AuthorDoc) -> Self {
        Self {
            message_id: value._id,
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct ProfileDocId {
    name: String,
    user_id: String,
}

#[derive(Deserialize, Serialize)]
struct ProfileDoc {
    _id: ProfileDocId,
    display_name: Option<String>,
    avatar: Option<String>,
    colour: Option<String>,
}

impl From<Profile> for ProfileDoc {
    fn from(value: Profile) -> Self {
        Self {
            _id: ProfileDocId {
                name: value.name,
                user_id: value.user_id,
            },
            display_name: value.display_name,
            avatar: value.avatar,
            colour: value.colour,
        }
    }
}
impl From<ProfileDoc> for Profile {
    fn from(value: ProfileDoc) -> Self {
        Self {
            user_id: value._id.user_id,
            name: value._id.name,
            display_name: value.display_name,
            avatar: value.avatar,
            colour: value.colour,
        }
    }
}

pub struct DB {
    authors_col: Collection<AuthorDoc>,
    profiles_col: Collection<ProfileDoc>,
    user_profiles: RwLock<HashMap<String, HashMap<String, Profile>>>,
}

impl DB {
    pub async fn new(
        uri: &str,
        db_name: &str,
        authors_col: &str,
        profiles_col: &str,
    ) -> Result<DB, mongodb::error::Error> {
        let mut options = ClientOptions::parse(uri).await?;
        options.app_name = Some("MasqueradeBot".to_string());
        let client = Client::with_options(options)?;
        let db = client.database(db_name);
        let authors_col = db.collection(authors_col);
        let profiles_col = db.collection::<ProfileDoc>(profiles_col);
        let mut user_profiles: HashMap<String, HashMap<String, Profile>> = HashMap::new();

        let mut cursor = profiles_col.find(doc! {}, None).await?;
        while let Some(profile_doc) = cursor.try_next().await? {
            let profile: Profile = profile_doc.into();
            if !user_profiles.contains_key(&profile.user_id) {
                user_profiles.insert(profile.user_id.clone(), HashMap::new());
            }
            user_profiles
                .get_mut(&profile.user_id)
                .unwrap()
                .insert(profile.name.clone(), profile);
        }
        Ok(Self {
            authors_col,
            profiles_col,
            user_profiles: RwLock::new(user_profiles),
        })
    }

    pub async fn get_profile(&self, user_id: &str, profile_name: &str) -> Option<Profile> {
        let user_profiles = self.user_profiles.read().await;
        user_profiles
            .get(user_id)
            .and_then(|u| u.get(profile_name))
            .cloned()
    }

    pub async fn get_profiles(&self, user_id: &str) -> Option<Vec<Profile>> {
        let user_profiles = self.user_profiles.read().await;
        let profiles = user_profiles.get(user_id)?;
        let mut profiles: Vec<_> = profiles.values().cloned().collect();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Some(profiles)
    }

    pub async fn delete_profile(
        &self,
        user_id: &str,
        profile_name: &str,
    ) -> Result<Option<Profile>, Error> {
        let mut user_profiles = self.user_profiles.write().await;
        self.profiles_col
            .delete_one(
                doc! {"_id": {"name": profile_name, "user_id": user_id}},
                None,
            )
            .await?;
        if let Some(profiles) = user_profiles.get_mut(user_id) {
            let maybe_profile = profiles.remove(profile_name);
            if profiles.is_empty() {
                user_profiles.remove(user_id);
            }
            return Ok(maybe_profile);
        }
        Ok(None)
    }

    pub async fn save_profile(&self, user_id: &str, profile: Profile) -> Result<(), Error> {
        profile.validate()?;
        let mut user_profiles = self.user_profiles.write().await;
        let profile_doc: ProfileDoc = profile.clone().into();
        let filter = doc! {"_id": to_document(&profile_doc._id).unwrap()};
        let mut update = doc! {"$set": to_document(&profile_doc).unwrap()};
        update.remove("_id");
        let options = UpdateOptions::builder().upsert(true).build();
        self.profiles_col
            .update_one(filter, update, Some(options))
            .await?;
        user_profiles
            .entry(user_id.to_string())
            .or_default()
            .insert(profile.name.clone(), profile);
        Ok(())
    }

    pub async fn get_author(&self, message_id: &str) -> Result<Option<Author>, Error> {
        let maybe_doc = self
            .authors_col
            .find_one(doc! {"_id": message_id}, None)
            .await?;
        Ok(maybe_doc.map(|doc| doc.into()))
    }

    pub async fn set_author(&self, author: Author) -> Result<(), Error> {
        let author_doc: AuthorDoc = author.into();
        self.authors_col.insert_one(author_doc, None).await?;
        Ok(())
    }
}
