use deskc_amir::environment::AEnvironment;
use deskc_ids::CardId;

use super::{CardQueries, QueryResult};

pub(super) fn amir_environment(db: &dyn CardQueries, id: CardId) -> QueryResult<AEnvironment> {
    todo!()
}
