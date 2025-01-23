// use crate::Perform;
// use actix_web::web::Data;
// use tinyboards_api_common::{
//     data::TinyBoardsContext,
//     site::{GetModLog, GetModLogResponse},
//     utils::{check_private_instance, get_user_view_from_jwt_opt, is_mod_or_admin, blocking}
// };
// use tinyboards_db::{
//     models::site::site::Site,
//     ModLogActionType,
//     map_to_modlog_type,
// };
// use tinyboards_db_views_mod::structs::*;
// use tinyboards_utils::error::TinyBoardsError;

// #[async_trait::async_trait(?Send)]
// impl<'des> Perform<'des> for GetModLog {
//     type Response = GetModLogResponse;
//     type Route = ();

//     #[tracing::instrument(skip_all)]
//     async fn perform(
//         self,
//         context: &Data<TinyBoardsContext>,
//         _: Self::Route,
//         auth: Option<&str>,
//     ) -> Result<Self::Response, TinyBoardsError> {
//         let data: &GetModLog = &self;
//         let user_view =
//             get_user_view_from_jwt_opt(auth, context.pool(), context.master_key())
//             .await?;

//         let site = blocking(context.pool(), move |conn| {
//             Site::read_local(conn)
//         })
//         .await??;

//         if site.private_instance {
//             return Err(TinyBoardsError::from_message("site is private"));
//         }

        
        

//         todo!()
//     }
// }