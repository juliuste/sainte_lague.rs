#![forbid(unsafe_code, unstable_features, missing_docs)]
#![deny(
    warnings,
    unused_extern_crates,
    missing_copy_implementations,
    missing_debug_implementations
)]

//! A rust implementation of the **[Sainte-Laguë](https://en.wikipedia.org/wiki/Webster/Sainte-Lagu%C3%AB_method)** (also known as **Webster** or **Schepers**) method. Parliament seat allocation algorithm used in multiple countries such as Germany, Latvia, New Zealand etc…
//!
//! *Attention: Since some countries (like Latvia or Norway) use a modification of the algorithm instead of this vanilla version, you should check your country's electoral legislature. Furthermore, I don't take any responsibility for the accuracy of the calculated numbers, even though I'm pretty confident with my implementation.*

use rand::seq::SliceRandom;
use std::error;
use std::fmt;

/// Possible error cases of [`distribute`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DistributionError {
    /// A distribution couldn't be determined because multiple parties were tied for the last seat. You can tell [`distribute`] to make a draw in these situations to prevent this error case.
    Tied,

    /// The given seat count was not larger than zero.
    InvalidSeatCount,

    /// The given list of votes contained negative values.
    NegativeVotes,

    /// The given list of votes contained no values or the sum of all values was zero.
    NoVotes,
}

impl fmt::Display for DistributionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DistributionError::Tied => write!(
                f,
                "Tie detected, could only be resolved by randomly awarding a seat to one party."
            ),
            &DistributionError::InvalidSeatCount => {
                write!(f, "Invalid seat count, must be an integer larger than 0.")
            }
            &DistributionError::NegativeVotes => write!(
                f,
                "Invalid votes, all parties must have at least zero votes."
            ),
            &DistributionError::NoVotes => {
                write!(f, "Invalid votes, one party must have at least one vote.")
            }
        }
    }
}

impl error::Error for DistributionError {}

#[derive(Clone)]
struct PartyQuotient {
    party: usize,
    quotient: f64,
}

/// Calculate the **[Sainte-Laguë](https://en.wikipedia.org/wiki/Webster/Sainte-Lagu%C3%AB_method)** distribution for the given `votes` and a parliament of size `seat_count`. Note that while votes are usually restricted to integers in normal elections, this function expects floating point numbers, allowing additional use cases.
///
/// The `draw_on_tie` flag should be used to indicate if the method should randomly assign seats in case of a draw or return an error instead.
///
/// Check [`DistributionError`] for a list of all possible error cases.
///
/// # Examples
///
/// Note that, while you would usually use the absolute vote counts to calculate a distribution, as explained before, the method is not restricted to integer input, so you can also use relative vote shares, as in in this example:
///
/// ```
/// use sainte_lague::distribute;
///
/// let votes_german_bundestag_2013 = [41.5, 25.7, 8.6, 8.4];
/// let seats_german_bundestag_2013 = 631;
/// let draw_on_tie = false;
///
/// let distribution = distribute(&votes_german_bundestag_2013, &seats_german_bundestag_2013, &draw_on_tie);
/// let parliament: Vec<usize> = vec![311, 193, 64, 63];
/// assert_eq!(distribution, Ok(parliament));
/// ```
///
/// The `draw_on_tie` flag is relevant in case of a draw:
///
/// ```
/// use sainte_lague::{distribute, DistributionError};
///
/// let votes = [3.0, 3.0, 1.0];
/// let seats = 8;
///
/// let distribution_without_draw = distribute(&votes, &seats, &false);
/// assert_eq!(distribution_without_draw, Err(DistributionError::Tied));
///
/// let distribution_with_draw = distribute(&votes, &seats, &true);
/// let parliament_draw_possibility_a: Vec<usize> = vec![4, 3, 1];
/// let parliament_draw_possibility_b: Vec<usize> = vec![3, 4, 1];
/// assert_eq!(
///     [Ok(parliament_draw_possibility_a), Ok(parliament_draw_possibility_b)]
///         .iter()
///         .any(|x| x == &distribution_with_draw),
///     true
/// );
/// ```
pub fn distribute(
    votes: &[f64],
    seat_count: &usize,
    draw_on_tie: &bool,
) -> Result<Vec<usize>, DistributionError> {
    // @todo this is certainly far from an optimal implementation, it is just a copy of
    // https://github.com/juliuste/sainte-lague for now, which should at least work correctly

    // validate prerequisites
    if seat_count < &1 {
        return Err(DistributionError::InvalidSeatCount);
    }
    let has_negative_votes = votes.iter().any(|v| v < &0.0);
    if has_negative_votes {
        return Err(DistributionError::NegativeVotes);
    }
    let total_votes: f64 = votes.iter().sum();
    if total_votes == 0.0 {
        return Err(DistributionError::NoVotes);
    }

    let mut party_quotients: Vec<PartyQuotient> = votes
        .iter()
        .enumerate()
        .flat_map(|(i, v)| {
            let divisors = (1..=(seat_count.clone() as i64)).map(|d| (d as f64) - 0.5);
            return divisors.map(move |d| PartyQuotient {
                party: i,
                quotient: v / d,
            });
        })
        .collect();

    party_quotients.sort_by(|a, b| {
        b.quotient
            .partial_cmp(&a.quotient)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let last_winning_quotient = party_quotients
        .get(seat_count.clone() - 1)
        .map(|pq| pq.quotient)
        .unwrap_or(0.0);
    let mut winners: Vec<PartyQuotient> = party_quotients
        .iter()
        .filter(|pq| pq.quotient > last_winning_quotient)
        .cloned()
        .collect();
    let mut possible_winners: Vec<PartyQuotient> = party_quotients
        .iter()
        .filter(|pq| pq.quotient == last_winning_quotient)
        .cloned()
        .collect();

    // check if the "last" winner had the same quotient as the "first" loser, if so we need
    // to make a draw to resolve the tie or return an error
    let seats_too_many =
        (winners.len() as i64) + (possible_winners.len() as i64) - (seat_count.clone() as i64);

    if seats_too_many > 0 {
        if !draw_on_tie {
            return Err(DistributionError::Tied);
        }
        let number_of_draws = (possible_winners.len() as i64) - seats_too_many;
        let mut drawn_winners: Vec<PartyQuotient> = (&possible_winners)
            .choose_multiple(&mut rand::thread_rng(), number_of_draws.max(0) as usize)
            .cloned()
            .collect();
        winners.append(&mut drawn_winners);
    } else {
        winners.append(&mut possible_winners);
    }

    let mut distribution: Vec<usize> = vec![0; votes.len()];
    for pq in winners.iter() {
        distribution[pq.party] += 1 // @todo
    }

    return Ok(distribution);
}

#[cfg(test)]
mod tests {
    use super::distribute;
    use super::DistributionError;

    #[test]
    fn german_bundestag_2013() {
        let votes = [41.5, 25.7, 8.6, 8.4];
        let seats = 631;

        let distribution = distribute(&votes, &seats, &false);
        let parliament = vec![311, 193, 64, 63];
        assert_eq!(distribution, Ok(parliament));
    }

    #[test]
    fn rhineland_palatinate_201x() {
        let votes = [362.0, 318.0, 126.0, 62.0, 53.0];
        let seats = 101;

        let distribution = distribute(&votes, &seats, &false);
        let parliament = vec![39, 35, 14, 7, 6];
        assert_eq!(distribution, Ok(parliament));
    }

    #[test]
    fn schleswig_holstein_201x() {
        let votes = [308.0, 304.0, 132.0, 82.0, 82.0, 46.0];
        let seats = 69;

        let distribution = distribute(&votes, &seats, &false);
        let parliament = vec![22, 22, 10, 6, 6, 3];
        assert_eq!(distribution, Ok(parliament));
    }

    #[test]
    fn equal_but_no_draw_required() {
        let votes = [415.0, 257.0, 85.0, 85.0];
        let seats = 631;

        let distribution = distribute(&votes, &seats, &false);
        let parliament = vec![311, 192, 64, 64];
        assert_eq!(distribution, Ok(parliament));
    }

    #[test]
    fn equal_and_draw_required() {
        let votes = [3.0, 3.0, 1.0];
        let seats = 8;

        let distribution_without_draw = distribute(&votes, &seats, &false);
        assert_eq!(distribution_without_draw, Err(DistributionError::Tied));

        let distribution_with_draw = distribute(&votes, &seats, &true);
        let parliament_draw_a: Vec<usize> = vec![4, 3, 1];
        let parliament_draw_b: Vec<usize> = vec![3, 4, 1];
        assert_eq!(
            [Ok(parliament_draw_a), Ok(parliament_draw_b)]
                .iter()
                .any(|x| x == &distribution_with_draw),
            true
        );
    }

    #[test]
    fn small_parliament_no_draw_required() {
        let votes = [2.0, 2.0];
        let seats = 2;

        let distribution_without_draw = distribute(&votes, &seats, &false);
        assert_eq!(distribution_without_draw, Ok(vec![1, 1]));

        let distribution_with_draw = distribute(&votes, &seats, &true);
        assert_eq!(distribution_with_draw, Ok(vec![1, 1]));
    }

    #[test]
    fn only_one_party() {
        let votes = [3.0];
        let seats = 10;

        let distribution = distribute(&votes, &seats, &false);
        let parliament = vec![10];
        assert_eq!(distribution, Ok(parliament));
    }

    #[test]
    fn invalid_seat_count() {
        let votes = [3.0];
        let seats = 0;

        let distribution = distribute(&votes, &seats, &false);
        assert_eq!(distribution, Err(DistributionError::InvalidSeatCount));
    }

    #[test]
    fn no_votes() {
        let seats = 50;

        let distribution_empty_votes = distribute(&[], &seats, &false);
        assert_eq!(distribution_empty_votes, Err(DistributionError::NoVotes));

        let distribution_zero_votes = distribute(&[0.0, 0.0], &seats, &false);
        assert_eq!(distribution_zero_votes, Err(DistributionError::NoVotes));

        let distribution_valid_votes = distribute(&[0.0, 3.0], &seats, &false);
        assert_eq!(distribution_valid_votes, Ok(vec![0, seats]));
    }

    #[test]
    fn negative_votes() {
        let seats = 50;

        let distribution_negative_votes = distribute(&[-3.0], &seats, &false);
        assert_eq!(
            distribution_negative_votes,
            Err(DistributionError::NegativeVotes)
        );

        let distribution_negative_votes_sum_zero = distribute(&[4.0, -4.0], &seats, &false);
        assert_eq!(
            distribution_negative_votes_sum_zero,
            Err(DistributionError::NegativeVotes)
        );
    }
}
