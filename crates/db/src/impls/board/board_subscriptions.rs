use crate::{
    models::board::board_subscriptions::{BoardSubscriber, BoardSubscriberForm},
    traits::Subscribeable,
    SubscribedType,
};
use diesel::{insert_into, result::Error, *};

impl BoardSubscriber {
    pub fn to_subscribed_type(subscriber: &Option<Self>) -> SubscribedType {
        match subscriber {
            Some(f) => {
                if f.pending.unwrap_or(false) {
                    SubscribedType::Pending
                } else {
                    SubscribedType::Subscribed
                }
            }
            None => SubscribedType::NotSubscribed,
        }
    }
}

impl Subscribeable for BoardSubscriber {
    type Form = BoardSubscriberForm;

    fn subscribe(conn: &mut diesel::PgConnection, sub_form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::board_subscriptions::dsl::{board_id, board_subscriptions, user_id};
        insert_into(board_subscriptions)
            .values(sub_form)
            .on_conflict((board_id, user_id))
            .do_update()
            .set(sub_form)
            .get_result::<Self>(conn)
    }

    fn unsubscribe(conn: &mut diesel::PgConnection, sub_form: &Self::Form) -> Result<usize, Error> {
        use crate::schema::board_subscriptions::dsl::{board_id, board_subscriptions, user_id};
        diesel::delete(
            board_subscriptions
                .filter(board_id.eq(sub_form.board_id))
                .filter(user_id.eq(sub_form.user_id)),
        )
        .execute(conn)
    }
}
