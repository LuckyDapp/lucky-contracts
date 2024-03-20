use crate::traits::raffle::RaffleError::*;
use crate::traits::RAFFLE_MANAGER_ROLE;
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, AccessControlError};
use openbrush::traits::{AccountId, Balance, Storage};
use scale::Decode;

use phat_rollup_anchor_ink::traits::rollup_anchor::RollupAnchor;
use scale::Encode;

const NEXT_ERA: u32 = ink::selector_id!("NEXT_ERA");
const NB_WINNERS: u32 = ink::selector_id!("NB_WINNERS");

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    ratio_distribution: Vec<Balance>,
    total_ratio_distribution: Balance,
    last_era_done: u32,
}

#[openbrush::trait_definition]
pub trait Raffle: Storage<Data> + access_control::Internal + RollupAnchor {
    /// Set the rate sharing by the winners
    /// First winner will receive : total_rewards * ratio[0] / total_ratio
    /// Second winner will receive : total_rewards * ratio[1] / total_ratio
    /// if ratio[n] equals to zero or is empty, tne winner n will receive nothing
    /// Sum(ratio[i]) <= total_ratio. Otherwise teh error IncorrectRatio is expected
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn set_ratio_distribution(
        &mut self,
        ratio: Vec<Balance>,
        total_ratio: Balance,
    ) -> Result<(), RaffleError> {
        let mut total = 0;
        for r in &ratio {
            total = r.checked_add(total).ok_or(AddOverFlow)?;
        }
        if total > total_ratio {
            return Err(IncorrectRatio);
        }

        self.data::<Data>().ratio_distribution = ratio;
        self.data::<Data>().total_ratio_distribution = total_ratio;

        // save the NB WINNERS in the kv store
        let nb_winners: u16 = self.data::<Data>().ratio_distribution.len() as u16;
        RollupAnchor::set_value(self, &NB_WINNERS.encode(), Some(&nb_winners.encode()));

        Ok(())
    }

    #[ink(message)]
    fn get_ratio_distribution(&self) -> Vec<Balance> {
        let ratio = &self.data::<Data>().ratio_distribution;
        ratio.to_vec()
    }

    #[ink(message)]
    fn get_total_ratio_distribution(&self) -> Balance {
        self.data::<Data>().total_ratio_distribution
    }

    #[ink(message)]
    fn get_next_era(&self) -> Result<u32, RaffleError> {
        match RollupAnchor::get_value(self, NEXT_ERA.encode()) {
            Some(v) => u32::decode(&mut v.as_slice()).map_err(|_| RaffleError::FailedToDecode),
            _ => Ok(0),
        }
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn set_next_era(&mut self, next_era: u32) -> Result<(), RaffleError> {
        self.inner_set_next_era(next_era)
    }

    fn inner_set_next_era(&mut self, next_era: u32) -> Result<(), RaffleError> {
        RollupAnchor::set_value(self, &NEXT_ERA.encode(), Some(&next_era.encode()));
        Ok(())
    }

    fn skip_raffle(&mut self, era: u32) -> Result<(), RaffleError> {
        // check if the raffle has not been done
        if self.get_next_era()? != era {
            return Err(IncorrectEra);
        }

        // set the raffle is done or skipped
        self.inner_set_next_era(era + 1)?;

        Ok(())
    }

    fn mark_raffle_done(
        &mut self,
        era: u32,
        total_rewards: Balance,
        winners: &Vec<AccountId>,
    ) -> Result<Vec<(AccountId, Balance)>, RaffleError> {
        // check if the raffle has not been done
        if self.get_next_era()? != era {
            return Err(IncorrectEra);
        }

        if total_rewards == 0 {
            // no reward
            return Err(NoReward);
        }

        let nb_winners = winners.len();

        if nb_winners == 0 {
            // no winner
            return Err(NoWinner);
        }

        let nb_ratio = self.data::<Data>().ratio_distribution.len();

        if nb_ratio == 0 {
            // no ration set
            return Err(NoRatioSet);
        }

        if nb_ratio < nb_winners {
            // no enough reward for all winners
            return Err(TooManyWinners);
        }

        let mut winners_and_rewards = Vec::with_capacity(nb_winners);

        for (i, winner) in winners.iter().enumerate() {
            // select the rewards ratio
            let ratio = self.data::<Data>().ratio_distribution.get(i).unwrap_or(&0);
            if *ratio != 0 {
                // compute the reward for this winner based on the ratio
                let amount = total_rewards
                    .checked_mul(*ratio)
                    .ok_or(MulOverFlow)?
                    .checked_div(self.data::<Data>().total_ratio_distribution)
                    .ok_or(DivByZero)?;
                // add the pending rewards for this account
                winners_and_rewards.push((*winner, amount));
            }
        }

        // set the raffle is done
        self.inner_set_next_era(era + 1)?;

        Ok(winners_and_rewards)
    }
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleError {
    IncorrectEra,
    NoReward,
    NoRatioSet,
    IncorrectRatio,
    NoWinner,
    TooManyWinners,
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    AccessControlError(AccessControlError),
    FailedToDecode,
}

/// convertor from AccessControlError to RaffleError
impl From<AccessControlError> for RaffleError {
    fn from(error: AccessControlError) -> Self {
        RaffleError::AccessControlError(error)
    }
}
