use crate::{
    models::user::user_mentions::{UserMention, UserMentionForm},
    traits::Crud,
};
use diesel::{result::Error, PgConnection, QueryDsl, RunQueryDsl};

impl Crud for UserMention {
    type Form = UserMentionForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::user_mentions::dsl::*;
        user_mentions.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::user_mentions::dsl::*;
        diesel::delete(user_mentions.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::user_mentions::dsl::*;
        let new = diesel::insert_into(user_mentions)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::user_mentions::dsl::*;
        diesel::update(user_mentions.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}