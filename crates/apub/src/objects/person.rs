use crate::{
    check_ap_id_valid_with_strictness,
    fetch_local_site_data, 
    protocol::{objects::{person::{Person, UserTypes}, Endpoints}, ImageObject, Source},
    objects::{instance::fetch_instance_actor_for_object, read_from_string_or_source_opt},
};
use tinyboards_federation::{
    config::Data,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::generate_outbox_url
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
    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>
    ) -> Result<Option<Self>, TinyBoardsError> {
        Ok(
            DbPerson::read_from_apub_id(context.pool(), &object_id.into())
            .await?
            .map(Into::into)
        )
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
            summary: self.bio.as_ref().map(|b| parse_markdown(b.as_str())),
            source: self.bio.clone().map(Source::new),
            icon: self.avatar.clone().map(ImageObject::new),
            image: self.banner.clone().map(ImageObject::new),
            matrix_user_id: None,
            published: Some(convert_datetime(self.creation_date)),
            outbox: generate_outbox_url(&self.actor_id)?.into(),
            endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
                shared_inbox: Url::parse(&s.to_string()).ok().unwrap(),
            }),
            public_key: self.public_key(),
            updated: self.updated.map(convert_datetime),
            inbox: Url::parse(&self.inbox_url.to_string()).ok().unwrap(),
        };

        Ok(person)
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        person: &Person,
        expected_domain: &Url,
        context: &Data<Self::DataType>
    ) -> Result<(), TinyBoardsError> {
        let local_site_data = fetch_local_site_data(context.pool()).await?;

        verify_domains_match(person.id.inner(), expected_domain)?;
        check_ap_id_valid_with_strictness(
            person.id.inner(), 
            false, 
            &local_site_data, 
            context.settings(),
        )?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn from_json(
        person: Person,
        context: &Data<Self::DataType>, 
    ) -> Result<ApubPerson, TinyBoardsError> {
        let instance_id = Some(fetch_instance_actor_for_object(&person.id, context).await?);
        
        // Some users have `name: ""`, need to convert that to `None`
        let display_name = person.name.filter(|n| !n.is_empty());

        // get instance name from person id
        let instance = person.id.inner().host_str().unwrap().to_owned();

        let person_form = PersonForm {
            name: Some(person.preferred_username),
            display_name,
            is_banned: Some(false),
            is_deleted: Some(false),
            avatar: person.icon.map(|i| i.url.into()),
            banner: person.image.map(|i| i.url.into()),
            creation_date: person.published.map(|u| u.naive_local()),
            updated: person.updated.map(|u| u.naive_local()),
            actor_id: Some(person.id.into()),
            bio: read_from_string_or_source_opt(&person.summary, &None, &person.source),
            local: Some(false),
            is_admin: Some(false),
            bot_account: Some(person.kind == UserTypes::Service),
            private_key: None,
            public_key: Some(person.public_key.public_key_pem),
            last_refreshed_date: Some(naive_now()),
            inbox_url: Some(person.inbox.into()),
            shared_inbox_url: person.endpoints.map(|e| e.shared_inbox.into()),
            instance_id,
            instance: Some(instance),
            ..PersonForm::default()
        };
        
        let person = DbPerson::upsert(context.pool(), &person_form).await?;
        
        Ok(person.into())
    }

}

impl Actor for ApubPerson {
    fn id(&self) -> Url {
        self.actor_id.clone().into()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }

    fn inbox(&self) -> Url {
        self.inbox_url.clone().into()
    }

    fn shared_inbox(&self) -> Option<Url> {
        self.shared_inbox_url.clone().map(Into::into)
    }
}

