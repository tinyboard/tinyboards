use crate::{
    check_ap_id_valid_with_strictness,
    fetch_local_site_data,
    config::Data
};

use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::TinyBoardsError;


// /// Try to fetch the instance actor (to make things like site rules available)
// pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
//     object_id: &T,
//     context: &Data<TinyBoardsContext>,
// ) -> Result<i32, TinyBoardsError> {
//     let object_id: Url = object_id.clone().into();
//     let instance_id = 
// }