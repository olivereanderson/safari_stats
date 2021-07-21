use std::{collections::HashMap, u8};

use rand::prelude::SliceRandom;
use rand_distr::{Binomial, Distribution};

use super::*;
pub(crate) struct TripData {
    pub(crate) session_id: Uuid,
    teams: (TeamData, TeamData),
}

impl TripData {
    pub(crate) fn into_participants_data(self) -> Vec<ParticipantData> {
        let (team1_data, team2_data) = (self.teams.0, self.teams.1);
        let participants: Vec<ParticipantData> = team1_data
            .participants
            .into_iter()
            .chain(team2_data.participants.into_iter())
            .collect();
        participants
    }
}

struct TeamData {
    participants: Vec<ParticipantData>,
}

impl TeamData {
    fn new(participants: Vec<ParticipantData>) -> Self {
        Self { participants }
    }
}

pub(crate) struct ParticipantData {
    pub(crate) user_id: Uuid,
    pub(crate) num_pics: u8,
    pub(crate) camera_id: u8,
}

impl ParticipantData {
    fn new(user_id: Uuid, num_pics: u8, camera_id: u8) -> Self {
        Self {
            user_id,
            num_pics,
            camera_id,
        }
    }
}

struct ParticipantToCamera {
    user_id_to_camera_id: HashMap<Uuid, u8>,
}
impl ParticipantToCamera {
    fn camera_id(&self, user_id: &Uuid) -> Option<u8> {
        self.user_id_to_camera_id.get(user_id).cloned()
    }
}

struct ParticipantToNumberOfPics {
    user_id_to_num_pics: HashMap<Uuid, u8>,
}
impl ParticipantToNumberOfPics {
    fn num_pics(&self, user_id: &Uuid) -> Option<u8> {
        self.user_id_to_num_pics.get(user_id).cloned()
    }
}

pub(crate) fn generate_trip_data<T: Rng>(synthetic_session: &Session, rng: &mut T) -> TripData {
    let (team1_user_ids, team2_user_ids) = (
        &synthetic_session.teams.0.user_ids[..],
        &synthetic_session.teams.1.user_ids[..],
    );
    // if we want to we could randomly drop some users from the two teams here before proceeding.

    let participant_to_camera = gen_camera_choices(team1_user_ids, team2_user_ids, rng);
    let participant_to_num_pics = gen_pics_by_participants(
        team1_user_ids,
        team2_user_ids,
        &participant_to_camera,
        rng,
    );
    let gen_team_data = |user_ids: &[Uuid]| -> Vec<ParticipantData> {
        let participant_data: Vec<ParticipantData> = user_ids
            .iter()
            .map(|id| {
                let camera_id = participant_to_camera.camera_id(id).unwrap();
                let num_pics = participant_to_num_pics.num_pics(id).unwrap();
                ParticipantData::new(*id, num_pics, camera_id)
            })
            .collect();
        participant_data
    };
    let team1_data = TeamData::new(gen_team_data(team1_user_ids));
    let team2_data = TeamData::new(gen_team_data(team2_user_ids));
    TripData {
        session_id: synthetic_session.id,
        teams: (team1_data, team2_data),
    }
}

// Returns a mapping taking a user's id to the id of the camera of their choice.
fn gen_camera_choices<T: Rng>(
    team1_user_ids: &[Uuid],
    team2_user_ids: &[Uuid],
    rng: &mut T,
) -> ParticipantToCamera {
    const NUM_OPERATORS: u8 = 100;
    let mut camera_ids: Vec<u8> = (1..=NUM_OPERATORS).collect();
    camera_ids.shuffle(rng); // todo find a better distribution
    let user_id_to_camera_id: HashMap<Uuid, u8> = team1_user_ids
        .iter()
        .chain(team2_user_ids.iter())
        .cloned()
        .zip(camera_ids.into_iter())
        .collect();
    ParticipantToCamera {
        user_id_to_camera_id,
    }
}

// Returns a mapping taking a user's id to a tuple where the first coordinate is their camera choice and the second
// is the number of pics this trip.
fn gen_pics_by_participants<T: Rng>(
    team1_user_ids: &[Uuid],
    team2_user_ids: &[Uuid],
    _camera_choices: &ParticipantToCamera,
    rng: &mut T,
) -> ParticipantToNumberOfPics {
    let mut user_id_to_num_pics: HashMap<Uuid, u8> =
        HashMap::with_capacity(team1_user_ids.len() + team2_user_ids.len());
    // We populate this mapping with one team at a time. We introduce a closure to do this.

    let generate_pics_by_team =
        |user_ids: &[Uuid], mapping: &mut HashMap<Uuid, u8>, rng: &mut T| {
            // The maximum number of total pics per team in a trip is 5 so we need to keep track of this
            let mut sum_pics_by_team = 0;
            // We use the binomial distrobution. The true distribution probably depends on the users in the trip and the cameras they chose.
            // todo: At least incorporate the choice of camera into the generation.
            let bin = Binomial::new(5, 0.1).unwrap();
            for id in user_ids.iter().cloned() {
                if sum_pics_by_team < 5 {
                    let mut num_pics = bin.sample(rng) as u8;
                    while num_pics + sum_pics_by_team > 5 {
                        num_pics = bin.sample(rng) as u8;
                    }
                    mapping.insert(id, num_pics);
                    sum_pics_by_team += num_pics;
                } else {
                    mapping.insert(id, 0);
                }
            }
        };
    generate_pics_by_team(team1_user_ids, &mut user_id_to_num_pics, rng);
    generate_pics_by_team(team2_user_ids, &mut user_id_to_num_pics, rng);
    ParticipantToNumberOfPics {
        user_id_to_num_pics,
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::session_generation::Team;
    use super::*;
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    // todo: take a mutable iterator over u128 as an argument and use that to generate ids
    fn mock_session() -> Session {
        let id = Uuid::from_u128(1);
        let teams = mock_teams();
        Session { id, teams }
    }
    // todo: take a mutable iterator over u128 as an argument and use that to generate ids rather than hardcoding them.
    fn mock_teams() -> (Team, Team) {
        let team1_user_ids = [
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
            Uuid::from_u128(4),
            Uuid::from_u128(5),
        ];
        let team2_user_ids = [
            Uuid::from_u128(6),
            Uuid::from_u128(7),
            Uuid::from_u128(8),
            Uuid::from_u128(9),
            Uuid::from_u128(10),
        ];
        let team1 = Team {
            user_ids: team1_user_ids,
        };
        let team2 = Team {
            user_ids: team2_user_ids,
        };
        (team1, team2)
    }
    #[test]
    fn some_pics() {
        // The probability of there not being a single pic in 100 trips should be extremely low. So we assume that this does not happen.
        let sessions: Vec<Session> = iter::repeat_with(mock_session).take(100).collect();
        let mut rng = Pcg64::seed_from_u64(2);
        //todo: Split this long chain into a few intermediate steps for the sake of code clarity.
        let total_num_pics: u32 = sessions
            .iter()
            .map(|x| generate_trip_data(x, &mut rng))
            .map(|x| x.into_participants_data())
            .map(|data| data.into_iter().map(|x| x.num_pics as u32))
            .map(|x| -> u32 { x.sum() })
            .sum();
        assert!(total_num_pics > 0);
    }

    #[test]
    fn team_never_pics_more_than_5_per_trip() {
        let sessions: Vec<Session> = iter::repeat_with(mock_session).take(100).collect();
        let mut rng = Pcg64::seed_from_u64(2);
        let first_team_trip_data_iter = sessions
            .iter()
            .map(|x| generate_trip_data(x, &mut rng))
            .map(|x| x.teams.0);
        let mut pics_by_first_team_per_trip_iter = first_team_trip_data_iter
            .map(|x| -> u32 { x.participants.iter().map(|x| x.num_pics as u32).sum() });
        assert!(!pics_by_first_team_per_trip_iter
            .any(|pics_by_team_this_trip| pics_by_team_this_trip > 5));
    }
}
