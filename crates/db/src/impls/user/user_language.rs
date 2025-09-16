use crate::{
    models::user::user_language::*,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

pub const UNDETERMINED_ID: i32 = 0;

impl UserLanguage {
    pub async fn read(
        pool: &DbPool,
        for_user_id: i32,
    ) -> Result<Vec<i32>, Error> {
        use crate::schema::user_language::dsl::{
            language_id,
            user_id,
            user_language
        };
        let conn = &mut get_conn(pool).await?;

        user_language
            .filter(user_id.eq(for_user_id))
            .order(language_id)
            .select(language_id)
            .get_results(conn)
            .await
    }

    /// Update the user's language preferences
    /// If no lang_id vector is given, show all langs
    pub async fn update(
        pool: &DbPool,
        language_ids: Vec<i32>,
        for_user_id: i32,
    ) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        let mut lang_ids = language_ids;

        // No need for an update if langs are unchanged
        let current = UserLanguage::read(pool, for_user_id).await?;
        if current == lang_ids {
            return Ok(());
        }

        // Always include undetermined language
        if !lang_ids.contains(&UNDETERMINED_ID) {
            lang_ids.push(UNDETERMINED_ID);
        }

        conn
            .build_transaction()
            .run(|conn| {
                Box::pin(async move {
                    use crate::schema::user_language::dsl::{
                        user_id, user_language,
                    };

                    // Delete existing user languages
                    delete(user_language.filter(user_id.eq(for_user_id)))
                        .execute(conn)
                        .await?;

                    // Insert new language preferences
                    for l in lang_ids {
                        let form = UserLanguageForm {
                            user_id: for_user_id,
                            language_id: l,
                        };

                        insert_into(user_language)
                            .values(form)
                            .get_result::<Self>(conn)
                            .await?;
                    }
                    Ok(())
                }) as _
            }).await
    }
}