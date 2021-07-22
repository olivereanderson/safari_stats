use common_utils::parsing_utils::Record;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UserRecord {
    pub(crate) user_id: Uuid,
    pub(crate) session_id: Uuid,
    pub(crate) nb_pics: u8,
}

impl From<Record> for UserRecord {
    fn from(record: Record) -> Self {
        Self {
            user_id: record.user_id,
            session_id: record.session_id,
            nb_pics: record.nb_pics,
        }
    }
}

impl UserRecord {
    pub(crate) fn split(self) -> (UserSessionRecord, u8) {
        (
            UserSessionRecord {
                user_id: self.user_id,
                session_id: self.session_id,
            },
            self.nb_pics,
        )
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub(crate) struct UserSessionRecord {
    pub(crate) user_id: Uuid,
    pub(crate) session_id: Uuid,
}
