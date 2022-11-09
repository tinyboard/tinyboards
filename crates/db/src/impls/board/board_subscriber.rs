use crate::{
    models::board::{
        board_subscriber::BoardSubscriber
    },
    SubscribedType,
};

impl BoardSubscriber {
    pub fn to_subscribed_type(subscriber: &Option<Self>) -> SubscribedType {
        match subscriber {
            Some(f) => {
                if f.pending.unwrap_or(false) {
                    SubscribedType::Pending
                } else {
                    SubscribedType::Subscribed
                }
            },
            None => SubscribedType::NotSubscribed,
        }
    }
}