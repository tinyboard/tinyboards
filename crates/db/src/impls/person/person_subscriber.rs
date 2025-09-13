use crate::{
    models::person::{person_subscriber::{PersonSubscriber, PersonSubscriberForm}, person::Person, user::User},
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

  async fn subscribe_accepted(
    pool: &DbPool,
    subscriber_id_: i32,
    person_id_: i32,
  ) -> Result<Self, Error> {
    use crate::schema::person_subscriber::dsl::{
      person_subscriber,
      subscriber_id,
      pending,
      person_id,
    };
    let conn = &mut get_conn(pool).await?;
    diesel::update(
        person_subscriber
          .filter(subscriber_id.eq(subscriber_id_))
          .filter(person_id.eq(person_id_)),
    )
    .set(pending.eq(false))
    .get_result::<Self>(conn)
    .await
  }
}

impl PersonSubscriber {
  /// Get list of people who are following the given person (followers)
  pub async fn list_subscribers(
    pool: &DbPool,
    for_person_id: i32,
  ) -> Result<Vec<User>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_ids: Vec<i32> = person_subscriber::table
      .filter(person_subscriber::person_id.eq(for_person_id))
      .filter(person_subscriber::pending.eq(false))
      .select(person_subscriber::subscriber_id)
      .load(conn)
      .await?;

    let mut users = Vec::new();
    for person_id in person_ids {
      if let Ok(user) = Person::get_user_by_id(pool, person_id).await {
        users.push(user);
      }
    }
    Ok(users)
  }

  /// Get list of people that the given person is following
  pub async fn list_following(
    pool: &DbPool,
    subscriber_id: i32,
  ) -> Result<Vec<User>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_ids: Vec<i32> = person_subscriber::table
      .filter(person_subscriber::subscriber_id.eq(subscriber_id))
      .filter(person_subscriber::pending.eq(false))
      .select(person_subscriber::person_id)
      .load(conn)
      .await?;

    let mut users = Vec::new();
    for person_id in person_ids {
      if let Ok(user) = Person::get_user_by_id(pool, person_id).await {
        users.push(user);
      }
    }
    Ok(users)
  }

  /// Get pending follow requests for a person
  pub async fn list_pending_follow_requests(
    pool: &DbPool,
    for_person_id: i32,
  ) -> Result<Vec<User>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_ids: Vec<i32> = person_subscriber::table
      .filter(person_subscriber::person_id.eq(for_person_id))
      .filter(person_subscriber::pending.eq(true))
      .select(person_subscriber::subscriber_id)
      .load(conn)
      .await?;

    let mut users = Vec::new();
    for person_id in person_ids {
      if let Ok(user) = Person::get_user_by_id(pool, person_id).await {
        users.push(user);
      }
    }
    Ok(users)
  }

  /// Check if a person is following another person
  pub async fn is_following(
    pool: &DbPool,
    subscriber_id: i32,
    person_id: i32,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    let result: Result<PersonSubscriber, Error> = person_subscriber::table
      .filter(person_subscriber::subscriber_id.eq(subscriber_id))
      .filter(person_subscriber::person_id.eq(person_id))
      .filter(person_subscriber::pending.eq(false))
      .first(conn)
      .await;
    
    Ok(result.is_ok())
  }
}