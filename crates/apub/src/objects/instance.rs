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
        apub::instance::Instance as DbInstance,
        site::*,
    },
    traits::Crud,
    utils::{naive_now, DbPool},
};
use tinyboards_utils::{
    parser::parse_markdown,
    time::convert_datetime,
};
use std::ops::Deref;
use tracing::debug;
use url::Url;



// /// Try to fetch the instance actor (to make things like site rules available)
// pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
//     object_id: &T,
//     context: &Data<TinyBoardsContext>,
// ) -> Result<i32, TinyBoardsError> {
//     let object_id: Url = object_id.clone().into();
//     let instance_id = 
// };