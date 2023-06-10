use tinyboards_db::{
    impls::apub::actor_language::UNDETERMINED_ID,
    models::apub::language::Language,
    utils::DbPool,
};
use tinyboards_utils::error::TinyBoardsError;
use serde::{Serialize, Deserialize};
use url::Url;

pub(crate) mod instance;
pub(crate) mod person;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Endpoints {
    pub shared_inbox: Url,
}

/// as specified at https://schema.org/Language
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageTag {
    pub(crate) identifier: String,
    pub(crate) name: String,
}

impl LanguageTag {
    pub(crate) async fn new_single(
        lang: i32,
        pool: &DbPool,
    ) -> Result<Option<LanguageTag>, TinyBoardsError> {
        let lang = Language::read_from_id(pool, lang).await?;

        // undetermined
        if lang.id == UNDETERMINED_ID {
            Ok(None)
        } else {
            Ok(Some(LanguageTag { 
                identifier: lang.code, 
                name: lang.name 
            }))
        }
    }

    pub(crate) async fn new_multiple(
        lang_ids: Vec<i32>,
        pool: &DbPool,
    ) -> Result<Vec<LanguageTag>, TinyBoardsError> {
        let mut langs = Vec::<Language>::new();

        for l in lang_ids {
            langs.push(Language::read_from_id(pool, l).await?);
        }

        let langs = langs
            .into_iter()
            .map(|l| LanguageTag {
                identifier: l.code,
                name: l.name,
            })
            .collect();
        Ok(langs)
    }

    pub(crate) async fn to_language_id_single(
        lang: Option<Self>,
        pool: &DbPool,
    ) -> Result<Option<i32>, TinyBoardsError> {
        let identifier = lang.map(|l| l.identifier);
        let language = Language::read_id_from_code(pool, identifier.as_deref()).await?;
        Ok(language)
    }

    pub(crate) async fn to_language_id_multiple(
        langs: Vec<Self>,
        pool: &DbPool,
    ) -> Result<Vec<Option<i32>>, TinyBoardsError> {
        let mut language_ids = Vec::new();

        for l in langs {
            let id = l.identifier;
            language_ids.push(Language::read_id_from_code(pool, Some(&id)).await?);
        }

        Ok(language_ids)

    }
}