use crate::traits::error::RaffleError;
use crate::traits::{Balance, RAFFLE_MANAGER_ROLE};
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use inkv5_client_lib::traits::access_control::{BaseAccessControl};
use inkv5_client_lib::traits::kv_store::KvStore;
use ink::env::DefaultEnvironment;
use ink::scale::{Decode, Encode};

const NEXT_ERA: u32 = ink::selector_id!("NEXT_ERA");
const NB_WINNERS: u32 = ink::selector_id!("NB_WINNERS");

#[derive(Default, Debug)]
#[ink::storage_item]
pub struct RaffleData {
    ratio_distribution: Vec<Balance>,
    total_ratio_distribution: Balance,
    last_era_done: u32,
}

#[ink::trait_definition]
pub trait Raffle {

    /// Set the rate sharing by the winners
    /// First winner will receive : total_rewards * ratio[0] / total_ratio
    /// Second winner will receive : total_rewards * ratio[1] / total_ratio
    /// if ratio[n] equals to zero or is empty, tne winner n will receive nothing
    /// Sum(ratio[i]) <= total_ratio. Otherwise teh error IncorrectRatio is expected
    #[ink(message)]
    //#[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn set_ratio_distribution(
        &mut self,
        ratio: Vec<Balance>,
        total_ratio: Balance,
    ) -> Result<(), RaffleError>;

    #[ink(message)]
    fn get_ratio_distribution(&self) -> Vec<Balance>;

    #[ink(message)]
    fn get_total_ratio_distribution(&self) -> Balance;

    #[ink(message)]
    fn get_next_era(&self) -> Result<u32, RaffleError>;

    #[ink(message)]
    //#[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn set_next_era(&mut self, next_era: u32) -> Result<(), RaffleError>;

}

pub trait RaffleStorage {
    fn get_storage(&self) -> &RaffleData;
    fn get_mut_storage(&mut self) -> &mut RaffleData;
}


pub trait BaseRaffle: RaffleStorage + KvStore + BaseAccessControl {

    /// Set the rate sharing by the winners
    /// First winner will receive : total_rewards * ratio[0] / total_ratio
    /// Second winner will receive : total_rewards * ratio[1] / total_ratio
    /// if ratio[n] equals to zero or is empty, tne winner n will receive nothing
    /// Sum(ratio[i]) <= total_ratio. Otherwise teh error IncorrectRatio is expected
    fn inner_set_ratio_distribution(
        &mut self,
        ratio: Vec<Balance>,
        total_ratio: Balance,
    ) -> Result<(), RaffleError> {

        let caller = ::ink::env::caller::<DefaultEnvironment>();
        self.inner_check_role(RAFFLE_MANAGER_ROLE, caller)?;

        let mut total = 0;
        for r in &ratio {
            total = r.checked_add(total).ok_or(RaffleError::AddOverFlow)?;
        }
        if total > total_ratio {
            return Err(RaffleError::IncorrectRatio);
        }

        RaffleStorage::get_mut_storage(self).ratio_distribution = ratio;
        RaffleStorage::get_mut_storage(self).total_ratio_distribution = total_ratio;

        // save the NB WINNERS in the kv store
        let nb_winners: u16 = u16::try_from(RaffleStorage::get_storage(self).ratio_distribution.len())?;
        KvStore::inner_set_value(self, &NB_WINNERS.encode(), Some(&nb_winners.encode()));

        Ok(())
    }

    fn inner_get_ratio_distribution(&self) -> Vec<Balance> {
        let ratio = &RaffleStorage::get_storage(self).ratio_distribution;
        ratio.to_vec()
    }

    fn inner_get_total_ratio_distribution(&self) -> Balance {
        RaffleStorage::get_storage(self).total_ratio_distribution
    }

    fn inner_get_next_era(&self) -> Result<u32, RaffleError> {
        match KvStore::inner_get_value(self, &NEXT_ERA.encode()) {
            Some(v) => u32::decode(&mut v.as_slice()).map_err(|_| RaffleError::FailedToDecode),
            _ => Ok(0),
        }
    }

    fn inner_set_next_era(&mut self, next_era: u32) -> Result<(), RaffleError> {
        let caller = ::ink::env::caller::<DefaultEnvironment>();
        self.inner_check_role(RAFFLE_MANAGER_ROLE, caller)?;

        KvStore::inner_set_value(self, &NEXT_ERA.encode(), Some(&next_era.encode()));
        Ok(())
    }

    fn skip_raffle(&mut self, era: u32) -> Result<(), RaffleError> {
        // check if the raffle has not been done
        if self.inner_get_next_era()? != era {
            return Err(RaffleError::IncorrectEra);
        }

        // set the raffle is done or skipped
        self.inner_set_next_era(era.checked_add(1).ok_or(RaffleError::AddOverFlow)?)?;

        Ok(())
    }

    fn mark_raffle_done(
        &mut self,
        era: u32,
        total_rewards: Balance,
        winners: &[AccountId],
    ) -> Result<Vec<(AccountId, Balance)>, RaffleError> {
        // check if the raffle has not been done
        if self.inner_get_next_era()? != era {
            return Err(RaffleError::IncorrectEra);
        }

        if total_rewards == 0 {
            // no reward
            return Err(RaffleError::NoReward);
        }

        let nb_winners = winners.len();

        if nb_winners == 0 {
            // no winner
            return Err(RaffleError::NoWinner);
        }

        let nb_ratio = RaffleStorage::get_storage(self).ratio_distribution.len();

        if nb_ratio == 0 {
            // no ration set
            return Err(RaffleError::NoRatioSet);
        }

        if nb_ratio < nb_winners {
            // no enough reward for all winners
            return Err(RaffleError::TooManyWinners);
        }

        let mut winners_and_rewards = Vec::with_capacity(nb_winners);

        for (i, winner) in winners.iter().enumerate() {
            // select the rewards ratio
            let ratio = RaffleStorage::get_storage(self).ratio_distribution.get(i).unwrap_or(&0);
            if *ratio != 0 {
                // compute the reward for this winner based on the ratio
                let amount = total_rewards
                    .checked_mul(*ratio)
                    .ok_or(RaffleError::MulOverFlow)?
                    .checked_div(RaffleStorage::get_storage(self).total_ratio_distribution)
                    .ok_or(RaffleError::DivByZero)?;
                // add the pending rewards for this account
                winners_and_rewards.push((*winner, amount));
            }
        }

        // set the raffle is done
        self.inner_set_next_era(era.checked_add(1).ok_or(RaffleError::AddOverFlow)?)?;

        Ok(winners_and_rewards)
    }
}
