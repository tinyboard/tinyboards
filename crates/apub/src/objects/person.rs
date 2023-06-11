use crate::{
    check_ap_id_valid_with_strictness,
    fetch_local_site_data, 
    protocol::{objects::{person::{Person, UserTypes}, Endpoints}, ImageObject, Source},
};
use tinyboards_federation::{
    config::Data,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::{generate_outbox_url}
};
use tinyboards_db::{
    models::person::person::{Person as DbPerson, PersonForm},
    traits::{ApubActor, Crud},
    utils::naive_now,
};
use tinyboards_utils::{
    error::TinyBoardsError,
    parser::parse_markdown,
    time::convert_datetime,
};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApubPerson(pub(crate) DbPerson);

impl Deref for ApubPerson {
    type Target = DbPerson;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DbPerson> for ApubPerson {
    fn from(p: DbPerson) -> Self {
        ApubPerson(p)
    }
}

#[async_trait::async_trait]
impl Object for ApubPerson {
    type DataType = TinyBoardsContext;
    type Kind = Person;
    type Error = TinyBoardsError;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_refreshed_date)
    }

    #[tracing::instrument(skip_all)]
    async fn delete(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        let form = PersonForm { is_deleted: Some(true), ..PersonForm::default() };
        DbPerson::update(context.pool(), self.id, &form).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn into_json(self, _context: &Data<Self::DataType>) -> Result<Person, TinyBoardsError> {
        let kind = if self.bot_account {
            UserTypes::Service
        } else {
            UserTypes::Person
        };

        let person = Person {
            kind,
            id: self.actor_id.clone().into(),
            preferred_username: self.name.clone(),
            name: self.display_name.clone(),
            summary: Some(self.bio.map(|b| parse_markdown(b.as_str())).unwrap_or_else(|| None).unwrap_or_else(|| String::new())),
            source: self.bio.clone().map(Source::new),
            icon: self.avatar.clone().map(ImageObject::new),
            image: self.banner.clone().map(ImageObject::new),
            matrix_user_id: None,
            published: Some(convert_datetime(self.creation_date)),
            outbox: generate_outbox_url(&self.actor_id)?.into(),
            endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
                shared_inbox: Url::parse(&s).ok().unwrap(),
            }),
            public_key: self.public_key(),
            updated: self.updated.map(convert_datetime),
            inbox: Url::parse(&self.inbox_url.clone()).ok().unwrap().into(),
        };

        Ok(person)
    }
}

impl Actor for ApubPerson {
    fn id(&self) -> Url {
        self.actor_id.inner().clone()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        &self.private_key.clone()
    }

    fn inbox(&self) -> Url {
        self.inbox_url.clone().into()
    }

    fn shared_inbox(&self) -> Option<Url> {
        self.shared_inbox_url.clone().map(Into::into)
    }
}

