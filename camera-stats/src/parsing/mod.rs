// todo: Consider moving this to its own crate

use common_utils::parsing_utils::Record;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CameraRecord {
    pub(crate) camera_id: u8,
    pub(crate) session_id: Uuid,
    pub(crate) nb_pics: u8,
}

impl From<Record> for CameraRecord {
    fn from(record: Record) -> Self {
        Self {
            camera_id: record.camera_id,
            session_id: record.session_id,
            nb_pics: record.nb_pics,
        }
    }
}
