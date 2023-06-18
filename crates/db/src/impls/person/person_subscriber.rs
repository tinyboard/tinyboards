use crate::{
    models::person::{person_subscriber::{PersonSubscriber, PersonSubscriberForm}, person::Person},
    traits::Subscribeable, utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl, ExpressionMethods, JoinOnDsl, insert_into};
use diesel_async::RunQueryDsl;
use crate::schema::{person, person_subscriber};

#[async_trait::async_trait]
impl Subscribeable for PersonSubscriber {
  type Form = PersonSubscriberForm;
  async fn subscribe(pool: &DbPool, form: &PersonSubscriberForm) -> Result<Self, Error> {
    use crate::schema::person_subscriber::dsl::{subscriber_id, person_subscriber, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(person_subscriber)
      .values(form)
      .on_conflict((subscriber_id, person_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn unsubscribe(pool: &DbPool, form: &PersonSubscriberForm) -> Result<usize, Error> {
    use crate::schema::person_subscriber::dsl::{subscriber_id, person_subscriber, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      person_subscriber
        .filter(subscriber_id.eq(&form.subscriber_id))
        .filter(person_id.eq(&form.person_id)),
    )
    .execute(conn)
    .await
  }
}

impl PersonSubscriber {
  pub async fn list_subscribers(
    pool: &DbPool,
    for_person_id: i32,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;
    person_subscriber::table
      .inner_join(person::table.on(person_subscriber::subscriber_id.eq(person::id)))
      .filter(person_subscriber::person_id.eq(for_person_id))
      .select(person::all_columns)
      .load(conn)
      .await
  }
}