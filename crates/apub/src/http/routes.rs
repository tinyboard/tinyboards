use crate::http::{
    comment::get_apub_comment,
    board::{
      board_inbox,
      get_apub_board_featured,
      get_apub_board_subscribers,
      get_apub_board_http,
      get_apub_board_moderators,
      get_apub_board_outbox,
    },
    get_activity,
    person::{get_apub_person_http, get_apub_person_outbox, person_inbox},
    post::get_apub_post,
    shared_inbox,
    site::{get_apub_site_http, get_apub_site_inbox, get_apub_site_outbox},
  };
use actix_web::{
    guard::{Guard, GuardContext},
    http::{header, Method},
    web,
};
use http_signature_normalization_actix::digest::middleware::VerifyDigest;
use sha2::{Digest, Sha256};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        // site routes
        .route("/", web::get().to(get_apub_site_http))
        .route("/site_outbox", web::get().to(get_apub_site_outbox))
        // board routes        
        .route("/+{board_name}", web::get().to(get_apub_board_http))
        .route("/+{board_name}/subscribers", web::get().to(get_apub_board_subscribers))
        .route("/+{board_name}/outbox", web::get().to(get_apub_board_outbox))
        .route("/+{board_name}/featured", web::get().to(get_apub_board_featured))
        .route("/+{board_name}/moderators", web::get().to(get_apub_board_moderators))
        // person (user) routes
        .route("/@{user_name}", web::get().to(get_apub_person_http))
        .route("/@{user_name}/outbox", web::get().to(get_apub_person_outbox))
        // post and comment routes
        .route("/post/{post_id}", web::get().to(get_apub_post))
        .route("/comment/{comment_id}", web::get().to(get_apub_comment))
        .route("/activities/{type_}/{id}", web::get().to(get_activity));

    cfg.service(
        web::scope("")
        .wrap(VerifyDigest::new(Sha256::new()))
        .guard(InboxRequestGuard)
        .route("/+{board_name}/inbox", web::post().to(board_inbox))
        .route("/@{user_name}/inbox", web::post().to(person_inbox))
        .route("/inbox", web::post().to(shared_inbox))
        .route("/site_inbox", web::post().to(get_apub_site_inbox)),
    );
}

/// Without this, things like webfinger or RSS feeds stop working, as all requests seem to get
/// routed into the inbox service (because it covers the root path). So we filter out anything that
/// definitely can't be an inbox request (based on Accept header and request method).
struct InboxRequestGuard;

impl Guard for InboxRequestGuard {
    fn check(&self, ctx: &GuardContext) -> bool {
      if ctx.head().method != Method::POST {
        return false;
      }
      if let Some(val) = ctx.head().headers.get(header::CONTENT_TYPE) {
        return val.as_bytes().starts_with(b"application/");
      }
      false
    }
}