// use crate::Perform;
// use actix_web::web::Data;
// use porpl_api_common::{
//     person::{Login, LoginResponse},
//     utils::{blocking, require_user},
//     data::PorplContext,
// };
// use porpl_utils::error::PorplError;




// #[async_trait::async_trait(?Send)]
// impl Perform for Login {
//     type Response = LoginResponse;

//     async fn perform(
//         &self,
//         context: &PorplContext,
//         authorization: Option<&str>,
//     ) -> Result<LoginResponse, PorplError> {
//         let data: &Login = self;

//         let name_or_email = data.username_or_email.clone();
//         let 
//     }
// }