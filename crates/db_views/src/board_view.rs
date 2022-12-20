use crate::structs::{BoardModeratorView, BoardView, UserView};
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::BoardAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber, board::boards::BoardSafe,
        board::user_board_blocks::BoardBlock, user::user::User,
    },
    schema::{board_aggregates, board_subscriptions, boards, user_board_blocks, users},
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, limit_and_offset},
    ListingType, SortType,
};
use typed_builder::TypedBuilder;

type BoardViewTuple = (
    BoardSafe,
    BoardAggregates,
    Option<BoardSubscriber>,
    Option<BoardBlock>,
);

impl BoardView {
    pub fn read(
        conn: &mut PgConnection,
        board_id: i32,
        user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = user_id.unwrap_or(-1);

        let (board, counts, subscriber, blocked) = boards::table
            .find(board_id)
            .inner_join(board_aggregates::table)
            .left_join(
                board_subscriptions::table.on(boards::id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                user_board_blocks::all_columns.nullable(),
            ))
            .first::<BoardViewTuple>(conn)?;

        Ok(BoardView {
            board,
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            blocked: blocked.is_some(),
            counts,
        })
    }

    pub fn is_admin(conn: &mut PgConnection, user_id: i32) -> Result<bool, Error> {
        let res = UserView::admins(conn)
            .map(|v| v.into_iter().map(|a| a.user.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&user_id);

        Ok(res)
    }

    pub fn is_mod_or_admin(conn: &mut PgConnection, user_id: i32, board_id: i32) -> bool {
        // check board moderators for user_id

        let is_mod = BoardModeratorView::for_board(conn, board_id)
            .map(|v| v.into_iter().map(|m| m.moderator.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&user_id);

        if is_mod {
            return true;
        }

        // check list of admins for user_id
        UserView::admins(conn)
            .map(|v| v.into_iter().map(|a| a.user.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&user_id)
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct BoardQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    listing_type: Option<ListingType>,
    sort: Option<SortType>,
    user: Option<&'a User>,
    search_term: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
}

impl<'a> BoardQuery<'a> {
    pub fn list(self) -> Result<Vec<BoardView>, Error> {
        let user_id_join = self.user.map(|l| l.id).unwrap_or(-1);

        let mut query = boards::table
            .inner_join(board_aggregates::table)
            .left_join(users::table.on(users::id.eq(user_id_join)))
            .left_join(
                board_subscriptions::table.on(boards::id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                user_board_blocks::all_columns.nullable(),
            ))
            .into_boxed();

        if let Some(search_term) = self.search_term {
            let searcher = fuzzy_search(&search_term);
            query = query
                .filter(boards::name.ilike(searcher.to_owned()))
                .or_filter(boards::title.ilike(searcher));
        }

        match self.sort.unwrap_or(SortType::Hot) {
            SortType::New => query = query.order_by(boards::published.desc()),
            SortType::TopAll => query = query.order_by(board_aggregates::subscribers.desc()),
            SortType::Hot => {
                query = query
                    .order_by(
                        hot_rank(board_aggregates::subscribers, board_aggregates::published).desc(),
                    )
                    .then_order_by(board_aggregates::published.desc());
            }
            _ => {
                query = query
                    .order_by(
                        hot_rank(board_aggregates::subscribers, board_aggregates::published).desc(),
                    )
                    .then_order_by(board_aggregates::published.desc())
            }
        };

        if let Some(listing_type) = self.listing_type {
            query = match listing_type {
                ListingType::Subscribed => query.filter(board_subscriptions::user_id.is_not_null()),
                ListingType::All => query,
            };
        }

        if self.user.is_some() {
            query = query.filter(user_board_blocks::user_id.is_null());
            query = query.filter(boards::nsfw.eq(false).or(users::show_nsfw.eq(true)));
        } else if !self.user.map(|l| l.show_nsfw).unwrap_or(false) {
            query = query.filter(boards::nsfw.eq(false));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .filter(boards::removed.eq(false))
            .filter(boards::deleted.eq(false))
            .load::<BoardViewTuple>(self.conn)?;

        Ok(BoardView::from_tuple_to_vec(res))
    }
}

impl ViewToVec for BoardView {
    type DbTuple = BoardViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                board: a.0,
                counts: a.1,
                subscribed: BoardSubscriber::to_subscribed_type(&a.2),
                blocked: a.3.is_some(),
            })
            .collect::<Vec<Self>>()
    }
}
