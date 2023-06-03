use crate::structs::{BoardModeratorView, BoardView, PersonView};
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::BoardAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber, board::boards::BoardSafe,
        board::person_board_blocks::BoardBlock, person::person::*,
    },
    schema::{board_aggregates, board_subscriptions, boards, person_board_blocks, person},
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, limit_and_offset, get_conn, DbPool},
    ListingType, SortType,
};
use typed_builder::TypedBuilder;
use diesel_async::RunQueryDsl;

type BoardViewTuple = (
    BoardSafe,
    BoardAggregates,
    Option<BoardSubscriber>,
    Option<BoardBlock>,
);

impl BoardView {
    pub async fn read(
        pool: &DbPool,
        board_id: i32,
        person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_id_join = person_id.unwrap_or(-1);

        let (board, counts, subscriber, blocked) = boards::table
            .find(board_id)
            .inner_join(board_aggregates::table)
            .left_join(
                board_subscriptions::table.on(boards::id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                person_board_blocks::all_columns.nullable(),
            ))
            .first::<BoardViewTuple>(conn)
            .await?;

        Ok(BoardView {
            board,
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            blocked: blocked.is_some(),
            counts,
        })
    }

    pub async fn is_admin(pool: &DbPool, person_id: i32) -> Result<bool, Error> {
        let res = UserView::admins(pool)
            .await
            .map(|v| v.into_iter().map(|a| a.user.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&person_id);

        Ok(res)
    }

    pub async fn is_mod_or_admin(pool: &DbPool, person_id: i32, board_id: i32) -> bool {
        // check board moderators for person_id
        let is_mod = BoardModeratorView::for_board(pool, board_id)
            .await    
            .map(|v| v.into_iter().map(|m| m.moderator.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&person_id);

        if is_mod {
            return true;
        }

        // check list of admins for person_id
        UserView::admins(pool)
            .await
            .map(|v| v.into_iter().map(|a| a.user.id).collect::<Vec<i32>>())
            .unwrap_or_default()
            .contains(&person_id)
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct BoardQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    listing_type: Option<ListingType>,
    sort: Option<SortType>,
    person: Option<&'a Person>,
    search_term: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct BoardQueryResponse {
    pub boards: Vec<BoardView>,
    pub count: i64,
}

impl<'a> BoardQuery<'a> {
    pub async fn list(self) -> Result<BoardQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        let person_id_join = self.person.map(|l| l.id).unwrap_or(-1);

        let mut query = boards::table
            .inner_join(board_aggregates::table)
            .left_join(person::table.on(person::id.eq(person_id_join)))
            .left_join(
                board_subscriptions::table.on(boards::id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                person_board_blocks::all_columns.nullable(),
            ))
            .into_boxed();

        let count_query = boards::table
            .inner_join(board_aggregates::table)
            .left_join(person::table.on(person::id.eq(person_id_join)))
            .left_join(
                board_subscriptions::table.on(boards::id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .select((BoardSafe::safe_columns_tuple(),))
            .into_boxed();

        if let Some(search_term) = self.search_term {
            let searcher = fuzzy_search(&search_term);
            query = query
                .filter(boards::name.ilike(searcher.to_owned()))
                .or_filter(boards::title.ilike(searcher));
        }

        match self.sort.unwrap_or(SortType::Hot) {
            SortType::New => query = query.order_by(boards::creation_date.desc()),
            SortType::TopAll => query = query.order_by(board_aggregates::subscribers.desc()),
            SortType::Hot => {
                query = query
                    .order_by(
                        hot_rank(
                            board_aggregates::subscribers,
                            board_aggregates::creation_date,
                        )
                        .desc(),
                    )
                    .then_order_by(board_aggregates::creation_date.desc());
            }
            _ => {
                query = query
                    .order_by(
                        hot_rank(
                            board_aggregates::subscribers,
                            board_aggregates::creation_date,
                        )
                        .desc(),
                    )
                    .then_order_by(board_aggregates::creation_date.desc())
            }
        };

        if let Some(listing_type) = self.listing_type {
            query = match listing_type {
                ListingType::Subscribed => query.filter(board_subscriptions::person_id.is_not_null()),
                ListingType::All => query,
            };
        }

        if self.person.is_some() {
            query = query.filter(person_board_blocks::person_id.is_null());
            query = query.filter(boards::is_nsfw.eq(false).or(person::show_nsfw.eq(true)));
        } else if !self.user.map(|l| l.show_nsfw).unwrap_or(false) {
            query = query.filter(boards::is_nsfw.eq(false));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .filter(boards::is_banned.eq(false))
            .filter(boards::is_deleted.eq(false))
            .load::<BoardViewTuple>(conn)
            .await?;

        let boards = BoardView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(BoardQueryResponse { boards, count })
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
