use crate::{
    models::board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    traits::Subscribeable,
    SubscribedType,
    utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, *};
use diesel_async::RunQueryDsl;

impl BoardSubscriber {
    pub fn to_subscribed_type(subscriber: &Option<Self>) -> SubscribedType {
        match subscriber {
            Some(f) => {
                if f.pending {
                    SubscribedType::Pending
                } else {
                    SubscribedType::Subscribed
                }
            }
            None => SubscribedType::NotSubscribed,
        }
    }
}

#[async_trait::async_trait]
impl Subscribeable for BoardSubscriber {
    type Form = BoardSubscriberForm;

    async fn subscribe(pool: &DbPool, sub_form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_subscriber::dsl::{board_id, board_subscriber, person_id};
        insert_into(board_subscriber)
            .values(sub_form)
            .on_conflict((board_id, person_id))
            .do_update()
            .set(sub_form)
            .get_result::<Self>(conn)
            .await
    }

    async fn unsubscribe(pool: &DbPool, sub_form: &Self::Form) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_subscriber::dsl::{board_id, board_subscriber, person_id};
        diesel::delete(
            board_subscriber
                .filter(board_id.eq(sub_form.board_id))
                .filter(person_id.eq(sub_form.person_id)),
        )
        .execute(conn)
        .await
    }
}
