use crate::{
    models::board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    traits::Subscribeable,
    utils::{get_conn, DbPool},
    SubscribedType,
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

    pub async fn subscribed_type_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        _for_user_id: i32,
    ) -> Result<Vec<(i32, SubscribedType)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{board_subscriber, boards};

        boards::table
            .left_join(board_subscriber::table.on(boards::id.eq(board_subscriber::board_id)))
            .filter(boards::id.eq_any(ids))
            .select((boards::id, board_subscriber::all_columns.nullable()))
            .load::<(i32, Option<BoardSubscriber>)>(conn)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(board_id, ref sub_opt)| (board_id, Self::to_subscribed_type(sub_opt)))
                    .collect::<Vec<(i32, SubscribedType)>>()
            })
    }
}

#[async_trait::async_trait]
impl Subscribeable for BoardSubscriber {
    type Form = BoardSubscriberForm;

    async fn subscribe(pool: &DbPool, sub_form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_subscriber::dsl::{board_id, board_subscriber, user_id};
        insert_into(board_subscriber)
            .values(sub_form)
            .on_conflict((board_id, user_id))
            .do_update()
            .set(sub_form)
            .get_result::<Self>(conn)
            .await
    }

    async fn unsubscribe(pool: &DbPool, sub_form: &Self::Form) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_subscriber::dsl::{board_id, board_subscriber, user_id};
        diesel::delete(
            board_subscriber
                .filter(board_id.eq(sub_form.board_id))
                .filter(user_id.eq(sub_form.user_id)),
        )
        .execute(conn)
        .await
    }

    async fn subscribe_accepted(
        pool: &DbPool,
        board_id_: i32,
        user_id_: i32,
    ) -> Result<Self, Error> {
        use crate::schema::board_subscriber::dsl::{
            board_id, board_subscriber, pending, user_id,
        };
        let conn = &mut get_conn(pool).await?;
        diesel::update(
            board_subscriber
                .filter(board_id.eq(board_id_))
                .filter(user_id.eq(user_id_)),
        )
        .set(pending.eq(false))
        .get_result::<Self>(conn)
        .await
    }
}
