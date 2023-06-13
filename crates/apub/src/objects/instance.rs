use crate::{
    check_ap_id_valid_with_strictness,
    fetch_local_site_data,
    objects::read_from_string_or_source_opt,
    protocol::{
        objects::{instance::Instance, LanguageTag},
        ImageObject,
        Source
    },
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::actor::ApplicationType,
    protocol::{values::MediaTypeHtml, verification::verify_domains_match},
    traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use tinyboards_api_common::{data::TinyBoardsContext};
use tinyboards_db::{
    models::{
        apub::actor_language::SiteLanguage,
        apub::{instance::Instance as DbInstance},
        site::{site::{Site, SiteForm}},
    },
    traits::Crud,
    utils::{naive_now, DbPool},
};
use tinyboards_utils::{
    parser::parse_markdown,
    time::convert_datetime,
    error::TinyBoardsError,
};
use std::ops::Deref;
use tracing::debug;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubSite(Site);

impl Deref for ApubSite {
    type Target = Site;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Site> for ApubSite {
    fn from(s: Site) -> Self {
        ApubSite(s)
    }
}

#[async_trait::async_trait]
impl Object for ApubSite {
    type DataType = TinyBoardsContext;
    type Kind = Instance;
    type Error = TinyBoardsError;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_refreshed_date)
    }

    #[tracing::instrument(skip_all)]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> {
        Ok(
            Site::read_from_apub_id(data.pool(), &object_id.into())
                .await?
                .map(Into::into)
        )
    }

    #[tracing::instrument(skip_all)]
    async fn delete(self, _data: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        unimplemented!()
    }

    #[tracing::instrument(skip_all)]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, TinyBoardsError> {
        let site_id = self.id;
        let langs = SiteLanguage::read(data.pool(), site_id).await?;
        let language = LanguageTag::new_multiple(langs, data.pool()).await?;

        let instance = Instance {
            kind: ApplicationType::Application,
            id: self.id().into(),
            name: self.name.clone(),
            content: self.sidebar.as_ref().map(|d| parse_markdown(d)).unwrap(),
            source: self.sidebar.clone().map(Source::new),
            summary: self.description.clone(),
            media_type: self.sidebar.as_ref().map(|_| MediaTypeHtml::Html),
            icon: self.icon.clone().map(ImageObject::new),
            image: self.banner.clone().map(ImageObject::new),
            inbox: self.inbox_url.clone().into(),
            outbox: Url::parse(&format!("{}/site_outbox", self.actor_id))?,
            public_key: self.public_key(),
            language,
            published: convert_datetime(self.creation_date),
            updated: Some(convert_datetime(self.updated.unwrap())),
        };
        Ok(instance)
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> Result<(), TinyBoardsError> {
        let local_site_data = fetch_local_site_data(data.pool()).await?;

        check_ap_id_valid_with_strictness(apub.id.inner(), true, &local_site_data, data.settings())?;
        verify_domains_match(expected_domain, apub.id.inner())?;

        Ok(())
    }
    
    #[tracing::instrument(skip_all)]
    async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, TinyBoardsError> {
      let domain = apub.id.inner().domain().expect("group id has domain");
      let instance = DbInstance::read_or_create(data.pool(), domain.to_string()).await?;
      
      let site_form = SiteForm {
        name: Some(apub.name.clone()),
        sidebar: Some(read_from_string_or_source_opt(&apub.content, &None, &apub.source)),
        updated: Some(apub.updated.map(|u| u.clone().naive_local())),
        icon: apub.icon.clone().map(|i| Some(i.url.into())),
        banner: apub.image.clone().map(|i| Some(i.url.into())),
        description: Some(apub.summary.clone()),
        actor_id: Some(apub.id.clone().into()),
        last_refreshed_date: Some(naive_now()),
        inbox_url: Some(apub.inbox.clone().into()),
        public_key: Some(apub.public_key.public_key_pem.clone()),
        private_key: None,
        instance_id: Some(instance.id),
        creation_date: Some(naive_now()),
      };
      let languages = LanguageTag::to_language_id_multiple(apub.language, data.pool()).await?;
  
      let site = Site::create(data.pool(), &site_form).await?;
      SiteLanguage::update(data.pool(), languages, &site).await?;
      Ok(site.into())
    }

}

impl Actor for ApubSite {
    fn id(&self) -> Url {
        self.actor_id.inner().clone()
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
}



/// Try to fetch the instance actor (to make things like site rules available)
pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
    object_id: &T,
    context: &Data<TinyBoardsContext>,
) -> Result<i32, TinyBoardsError> {
    let object_id: Url = object_id.clone().into();
    let instance_id = Site::instance_actor_id_from_url(object_id);
    let site = ObjectId::<ApubSite>::from(instance_id.clone())
        .dereference(context)
        .await;

    match site {
        Ok(s) => Ok(s.instance_id),
        Err(e) => {
            debug!("Failed to dereference site for {}: {}", &instance_id, e);
            let domain = instance_id.domain().expect("has domain");
            Ok(
                DbInstance::read_or_create(context.pool(), domain.to_string())
                    .await?
                    .id,
            )
        }
    }
}

pub(crate) async fn remote_instance_inboxes(pool: &DbPool) -> Result<Vec<Url>, TinyBoardsError> {
    Ok(
      Site::read_remote_sites(pool)
        .await?
        .into_iter()
        .map(|s| ApubSite::from(s).shared_inbox_or_inbox())
        .collect::<Vec<Url>>(),
    )
  }