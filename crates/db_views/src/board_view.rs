use crate::actor_structs::BoardView;
use diesel::{result::Error, *};
use porpl_db::{
    aggregates::structs::BoardAggregates,
    schema::{
        board,
        board_aggregates,
        board_block,
        board_subscriber,
        user_,
    },
    models::{
        board::board::BoardSafe,
        board::board_subscriber::BoardSubscriber,
        board::board_block::BoardBlock,
        user::user::User,
    },
    traits::{ToSafe, ViewToVec},
    ListingType,
    SortType, utils::{fuzzy_search, functions::hot_rank, limit_and_offset},
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

        let (board, counts, subscriber, blocked) = board::table
            .find(board_id)
            .inner_join(board_aggregates::table)
            .left_join(
                board_subscriber::table.on(
                    board::id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                board_block::table.on(
                    board::id
                        .eq(board_block::board_id)
                        .and(board_block::user_id.eq(user_id_join)),
                ),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                board_block::all_columns.nullable(),
            ))
            .first::<BoardViewTuple>(conn)?;
        
        Ok(BoardView {
            board,
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            blocked: blocked.is_some(),
            counts,
        })
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

        let mut query = board::table
            .inner_join(board_aggregates::table)
            .left_join(user_::table.on(user_::id.eq(user_id_join)))
            .left_join(
                board_subscriber::table.on(
                    board::id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                board_block::table.on(
                    board::id
                        .eq(board_block::board_id)
                        .and(board_block::user_id.eq(user_id_join)),
                ),
            )
            .select((
                BoardSafe::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                board_block::all_columns.nullable(),
            ))
            .into_boxed();

            if let Some(search_term) = self.search_term {
                let searcher = fuzzy_search(&search_term);
                query = query
                    .filter(board::name.ilike(searcher.to_owned()))
                    .or_filter(board::title.ilike(searcher));
            }

            match self.sort.unwrap_or(SortType::Hot) {
                SortType::New => query = query.order_by(board::published.desc()),
                SortType::TopAll => query = query.order_by(board_aggregates::subscribers.desc()),
                SortType::Hot => {
                    query = query
                        .order_by(
                            hot_rank(
                                board_aggregates::subscribers,
                                board_aggregates::published,
                            )
                            .desc(),
                        )
                        .then_order_by(board_aggregates::published.desc());
                }
                _ => {
                    query = query.order_by(
                        hot_rank(
                            board_aggregates::subscribers,
                            board_aggregates::published,
                        )
                        .desc(),
                    )
                    .then_order_by(board_aggregates::published.desc())
                }
            };

            if let Some(listing_type) = self.listing_type {
                query = match listing_type {
                    ListingType::Subscribed => query.filter(board_subscriber::user_id.is_not_null()),
                    ListingType::All => query,
                    _ => query,
                };
            }

            if self.user.is_some() {
                query = query.filter(board_block::user_id.is_null());
                query = query.filter(board::nsfw.eq(false).or(user_::show_nsfw.eq(true)));
            } else {
                if !self.user.map(|l| l.show_nsfw).unwrap_or(false) {
                    query = query.filter(board::nsfw.eq(false));
                }
            }

            let (limit, offset) = limit_and_offset(self.page, self.limit)?;

            let res = query
                .limit(limit)
                .offset(offset)
                .filter(board::removed.eq(false))
                .filter(board::deleted.eq(false))
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
