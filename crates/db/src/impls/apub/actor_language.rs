use diesel_async::pooled_connection::bb8::PooledConnection;
use tinyboards_utils::TinyBoardsError;
use tokio::sync::OnceCell;
use crate::models::apub::language::Language;
use crate::models::site::site::Site;
use crate::schema::{/*local_user_language,*/ site_language, /*board_language,*/ site, local_site};
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::actor_language::*,
    //traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::{
    AsyncPgConnection,
    RunQueryDsl,
};

pub const UNDETERMINED_ID: i32 = 0;

impl LocalUserLanguage {
    pub async fn read(
        pool: &DbPool,
        for_local_user_id: i32,
    ) -> Result<Vec<i32>, Error> {
        use crate::schema::local_user_language::dsl::{
            language_id,
            local_user_id,
            local_user_language
        };
        let conn = &mut get_conn(pool).await?;

        conn
            .build_transaction()
            .run(|conn| {
                Box::pin(async move {
                    let langs = local_user_language
                        .filter(local_user_id.eq(for_local_user_id))
                        .order(language_id)
                        .select(language_id)
                        .get_results(conn)
                        .await?;
                    convert_read_languages(conn, langs).await
                }) as _
            })
            .await
    }

    /// update the local user's languages
    /// 
    /// if no lang_id vector is given, show all langs
    pub async fn update(
        pool: &DbPool,
        language_ids: Vec<i32>,
        for_local_user_id: i32,
    ) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        let mut lang_ids = convert_update_languages(conn, language_ids).await?;

        // no need for an update if langs are unchanged
        let current = LocalUserLanguage::read(pool, for_local_user_id).await?;
        if current == lang_ids {
            return Ok(());
        }

        if !lang_ids.contains(&UNDETERMINED_ID) {
            lang_ids.push(UNDETERMINED_ID);
        }

        conn
            .build_transaction()
            .run(|conn| {
                Box::pin(async move {
                    use crate::schema::local_user_language::dsl::{
                        local_user_id, local_user_language,
                    };
                    delete(local_user_language.filter(local_user_id.eq(for_local_user_id)))
                        .execute(conn)
                        .await?;

                    for l in lang_ids {
                        let form = LocalUserLanguageForm {
                            local_user_id: for_local_user_id,
                            language_id: l,
                        };

                        insert_into(local_user_language)
                            .values(form)
                            .get_result::<Self>(conn)
                            .await?;
                    }
                    Ok(())
                }) as _
            }).await

    }
}

impl SiteLanguage {
    pub async fn read_local_raw(pool: &DbPool) -> Result<Vec<i32>, Error> {
      let conn = &mut get_conn(pool).await?;
      site::table
        .inner_join(local_site::table)
        .inner_join(site_language::table)
        .order(site_language::id)
        .select(site_language::language_id)
        .load(conn)
        .await
    }
  
    async fn read_raw(
      conn: &mut PooledConnection<'_, AsyncPgConnection>,
      for_site_id: i32,
    ) -> Result<Vec<i32>, Error> {
      site_language::table
        .filter(site_language::site_id.eq(for_site_id))
        .order(site_language::language_id)
        .select(site_language::language_id)
        .load(conn)
        .await
    }
  
    pub async fn read(pool: &DbPool, for_site_id: i32) -> Result<Vec<i32>, Error> {
      let conn = &mut get_conn(pool).await?;
      let langs = Self::read_raw(conn, for_site_id).await?;
  
      convert_read_languages(conn, langs).await
    }
  
    pub async fn update(
      pool: &DbPool,
      language_ids: Vec<i32>,
      site: &Site,
    ) -> Result<(), Error> {
      let conn = &mut get_conn(pool).await?;
      let for_site_id = site.id;
      let instance_id = site.instance_id;
      let lang_ids = convert_update_languages(conn, language_ids).await?;
  
      // No need to update if languages are unchanged
      let current = SiteLanguage::read(pool, site.id).await?;
      if current == lang_ids {
        return Ok(());
      }
  
      conn
        .build_transaction()
        .run(|conn| {
          Box::pin(async move {
            use crate::schema::site_language::dsl::{site_id, site_language};
  
            // Clear the current languages
            delete(site_language.filter(site_id.eq(for_site_id)))
              .execute(conn)
              .await?;
  
            for l in lang_ids {
              let form = SiteLanguageForm {
                site_id: for_site_id,
                language_id: l,
              };
              insert_into(site_language)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
  
            BoardLanguage::limit_languages(conn, instance_id).await?;
  
            Ok(())
          }) as _
        })
        .await
    }
  }

  impl BoardLanguage {
    /// Returns true if the given language is one of configured languages for given community
    pub async fn is_allowed_board_language(
      pool: &DbPool,
      for_language_id: Option<i32>,
      for_board_id: i32,
    ) -> Result<(), TinyBoardsError> {
      use crate::schema::board_language::dsl::{board_id, board_language, language_id};
      let conn = &mut get_conn(pool).await?;
  
      if let Some(for_language_id) = for_language_id {
        let is_allowed = select(exists(
          board_language
            .filter(language_id.eq(for_language_id))
            .filter(board_id.eq(for_board_id)),
        ))
        .get_result(conn)
        .await?;
  
        if is_allowed {
          Ok(())
        } else {
          Err(TinyBoardsError::from_message(500, "language not allowed"))
        }
      } else {
        Ok(())
      }
    }
  
    /// When site languages are updated, delete all languages of local boards which are not
    /// also part of site languages. This is because post/comment language is only checked against
    /// board language, and it shouldnt be possible to post content in languages which are not
    /// allowed by local site.
    async fn limit_languages(
      conn: &mut AsyncPgConnection,
      for_instance_id: i32,
    ) -> Result<(), Error> {
      use crate::schema::{
        boards::dsl as b,
        board_language::dsl as cl,
        site_language::dsl as sl,
      };
      let board_languages: Vec<i32> = cl::board_language
        .left_outer_join(sl::site_language.on(cl::language_id.eq(sl::language_id)))
        .inner_join(b::boards)
        .filter(b::instance_id.eq(for_instance_id))
        .filter(sl::language_id.is_null())
        .select(cl::language_id)
        .get_results(conn)
        .await?;
  
      for c in board_languages {
        delete(cl::board_language.filter(cl::language_id.eq(c)))
          .execute(conn)
          .await?;
      }
      Ok(())
    }
  
    async fn read_raw(
      conn: &mut PooledConnection<'_, AsyncPgConnection>,
      for_board_id: i32,
    ) -> Result<Vec<i32>, Error> {
      use crate::schema::board_language::dsl::{board_id, board_language, language_id};
      board_language
        .filter(board_id.eq(for_board_id))
        .order(language_id)
        .select(language_id)
        .get_results(conn)
        .await
    }
  
    pub async fn read(
      pool: &DbPool,
      for_board_id: i32,
    ) -> Result<Vec<i32>, Error> {
      let conn = &mut get_conn(pool).await?;
      let langs = Self::read_raw(conn, for_board_id).await?;
      convert_read_languages(conn, langs).await
    }
  
    pub async fn update(
      pool: &DbPool,
      mut language_ids: Vec<i32>,
      for_board_id: i32,
    ) -> Result<(), Error> {
      let conn = &mut get_conn(pool).await?;
      if language_ids.is_empty() {
        language_ids = SiteLanguage::read_local_raw(pool).await?;
      }
      let lang_ids = convert_update_languages(conn, language_ids).await?;
  
      // No need to update if languages are unchanged
      let current = BoardLanguage::read_raw(conn, for_board_id).await?;
      if current == lang_ids {
        return Ok(());
      }
  
      conn
        .build_transaction()
        .run(|conn| {
          Box::pin(async move {
            use crate::schema::board_language::dsl::{board_id, board_language};
            // Clear the current languages
            delete(board_language.filter(board_id.eq(for_board_id)))
              .execute(conn)
              .await?;
  
            for l in lang_ids {
              let form = BoardLanguageForm {
                board_id: for_board_id,
                language_id: l,
              };
              insert_into(board_language)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
            Ok(())
          }) as _
        })
        .await
    }
  }

  pub async fn default_post_language(
    pool: &DbPool,
    board_id: i32,
    local_user_id: i32,
  ) -> Result<Option<i32>, Error> {
    use crate::schema::{board_language::dsl as bl, local_user_language::dsl as ul};
    let conn = &mut get_conn(pool).await?;
    let mut intersection = ul::local_user_language
      .inner_join(bl::board_language.on(ul::language_id.eq(bl::language_id)))
      .filter(ul::local_user_id.eq(local_user_id))
      .filter(bl::board_id.eq(board_id))
      .select(bl::language_id)
      .get_results::<i32>(conn)
      .await?;
  
    if intersection.len() == 1 {
      Ok(intersection.pop())
    } else if intersection.len() == 2 && intersection.contains(&UNDETERMINED_ID) {
      intersection.retain(|i| i != &UNDETERMINED_ID);
      Ok(intersection.pop())
    } else {
      Ok(None)
    }
}
  
/// If no lang is given, set all langs
async fn convert_update_languages(
    conn: &mut AsyncPgConnection,
    language_ids: Vec<i32>,
  ) -> Result<Vec<i32>, Error> {
    if language_ids.is_empty() {
      Ok(
        Language::read_all_conn(conn)
          .await?
          .into_iter()
          .map(|l| l.id)
          .collect(),
      )
    } else {
      Ok(language_ids)
    }
  }


/// If all langs are returned, return empty vec instead
async fn convert_read_languages(
    conn: &mut AsyncPgConnection,
    language_ids: Vec<i32>,
) -> Result<Vec<i32>, Error> {
    static ALL_LANGUAGES_COUNT: OnceCell<usize> = OnceCell::const_new();
    let count = ALL_LANGUAGES_COUNT
        .get_or_init(|| async {
            use crate::schema::language::dsl::{id, language};
            let count: i64 = language
                .select(count(id))
                .first(conn)
                .await
                .expect("read number of languages");

            count as usize
        })
        .await;

    if &language_ids.len() == count {
        Ok(vec![])
    } else {
        Ok(language_ids)
    }
}