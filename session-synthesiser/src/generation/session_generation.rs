use super::*;

use std::{collections::HashSet, iter, u128};

pub(crate) struct Session {
    pub(super) id: Uuid,
    pub(super) teams: (Team, Team),
}

pub(super) struct Team {
    pub(super) user_ids: [Uuid; 5],
}
fn generate_teams<T: Rng>(rng: &mut T) -> (Team, Team) {
    const NUM_PLAYERS: u32 = 40_000_000;
    // todo: Find a more accurate distribution
    let distr = rand::distributions::Uniform::new_inclusive(1u32, NUM_PLAYERS);
    // generate 10 unique user ids.
    let mut user_ids: HashSet<u128> = iter::repeat_with(|| rng.sample(distr) as u128)
        .take(10)
        .collect();
    if user_ids.len() < 10 {
        // This means that the random generator did not generate 10 random numbers
        // the probability of this happening should be very low, so we try again.
        generate_teams(rng)
    } else {
        let mut draining_iter = user_ids.drain();
        let mut user_ids_team1 = [Uuid::default(); 5];
        let mut user_ids_team2 = [Uuid::default(); 5];
        for id in user_ids_team1
            .iter_mut()
            .chain(user_ids_team2.iter_mut())
        {
            *id = Uuid::from_u128(draining_iter.next().unwrap());
        }
        let team1 = Team {
            user_ids: user_ids_team1,
        };
        let team2 = Team {
            user_ids: user_ids_team2,
        };
        (team1, team2)
    }
}

pub(crate) fn generate_session<T: Rng>(rng: &mut T) -> Session {
    let id: u128 = rng.gen();
    //
    let teams = generate_teams(rng);
    Session {
        id: Uuid::from_u128(id),
        teams,
    }
}

pub(crate) fn generate_number_of_trips<T: Rng>(_synthetic_session: &Session, rng: &mut T) -> u8 {
    // todo: Consider making this function depend on the given session.
    // todo: Find a more accurate distribution.
    rng.gen_range(4..9)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    #[test]
    fn unique_user_ids_team_gen() {
        let mut rng = Pcg64::seed_from_u64(2);
        let teams = generate_session(&mut rng).teams;
        let ids: HashSet<Uuid> = teams
            .0
            .user_ids
            .iter()
            .chain(teams.1.user_ids.iter())
            .cloned()
            .collect();
        assert_eq!(10, ids.len());
    }
}
