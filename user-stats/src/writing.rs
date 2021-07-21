use itertools::Itertools;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(PartialEq, Debug)]
pub(crate) struct UserBestStats {
    pub(crate) user_id: Uuid,
    pub(crate) session_id_num_pics_pairs: Vec<(Uuid, u8)>,
}

impl Display for UserBestStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let write_string = std::iter::once(format!("{}|", self.user_id))
            .chain(
                self.session_id_num_pics_pairs
                    .iter()
                    .map(|(session_id, nb_pics)| format!("{}:{},", session_id, nb_pics)),
            )
            .join("");
        write!(f, "{}", write_string)
    }
}

impl Default for UserBestStats {
    fn default() -> Self {
        Self {
            user_id: Uuid::default(),
            session_id_num_pics_pairs: Vec::<(Uuid, u8)>::with_capacity(10),
        }
    }
}
