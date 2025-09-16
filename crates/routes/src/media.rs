use actix_web::web;
use tinyboards_api_graphql::utils::files::GetFile;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "media/{filename}",
        web::get().to(GetFile::perform)
    );
}
