use crate::{
    models::board::{
        board::Board
    },
    traits::{Crud, Followable, Joinable, Bannable},
    SubscribedType,
};
use diesel::{
    dsl::*,
    result::Error,
    ExpressionMethods,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
    TextExpressionMethods,
};

mod safe_type {
    use crate::{schema::board::*, models::board::board::Board, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        description,
        removed,
        published,
        updated,
        deleted,
        nsfw,
    );

    impl ToSafe for Board {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                description,
                removed,
                published,
                updated,
                deleted,
                nsfw,
            )
        }
    }
}

