use crate::{
    models::{
        post::post_saved::{PostSaved, PostSavedForm},
    },
    traits::{
        Saveable,
    }
};
use diesel::{
    prelude::*,
    PgConnection, 
    insert_into,
};
use porpl_utils::PorplError;

impl Saveable for PostSaved {
    type Form = PostSavedForm;
    fn save(conn: &mut PgConnection, form: &PostSavedForm) -> Result<Self, PorplError> {
        use crate::schema::post_saved::dsl::*;
        insert_into(post_saved)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            }
        )
    }

    fn unsave(conn: &mut PgConnection, form: &PostSavedForm) -> Result<usize, PorplError> {
        use crate::schema::post_saved::dsl::*;
        diesel::delete(
            post_saved
                .filter(post_id.eq(form.post_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })
    }
}