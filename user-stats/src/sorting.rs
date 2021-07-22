use crate::parsing::UserSessionRecord;
use rayon::prelude::*;
use std::ops::AddAssign;

pub(crate) fn sort_collect_splitted_user_records(
    record_pairs: &mut Vec<(UserSessionRecord, u8)>,
) {
    record_pairs.par_sort_unstable_by(|(record_x, _num_pics_x), (record_y, _num_pics_y)| {
        record_x.cmp(record_y)
    });
    collect_sorted_splitted_user_records(record_pairs);
}

fn collect_sorted_splitted_user_records(record_pairs: &mut Vec<(UserSessionRecord, u8)>) {
    record_pairs.dedup_by(|(record_x, num_pics_x), (record_y, num_pics_y)| {
        if record_y == record_x {
            num_pics_y.add_assign(*num_pics_x);
            true
        } else {
            false
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    impl UserSessionRecord {
        fn new(user_id: u128, session_id: u128) -> Self {
            Self {
                user_id: Uuid::from_u128(user_id),
                session_id: Uuid::from_u128(session_id),
            }
        }
    }
    #[test]
    fn sort_collect_splitted_user_records_works() {
        let mut user_session_records_nb_pics_pairs = vec![
            (UserSessionRecord::new(1, 100), 3u8),
            (UserSessionRecord::new(2, 200), 2u8),
            (UserSessionRecord::new(3, 100), 0u8),
            (UserSessionRecord::new(2, 200), 1u8),
            (UserSessionRecord::new(1, 100), 2u8),
            (UserSessionRecord::new(1, 500), 4u8),
        ];

        let sorted_and_collected_pairs = vec![
            (UserSessionRecord::new(1, 100), 5u8),
            (UserSessionRecord::new(1, 500), 4u8),
            (UserSessionRecord::new(2, 200), 3u8),
            (UserSessionRecord::new(3, 100), 0u8),
        ];
        sort_collect_splitted_user_records(&mut user_session_records_nb_pics_pairs);
        assert_eq!(
            sorted_and_collected_pairs,
            user_session_records_nb_pics_pairs
        );
    }
}
