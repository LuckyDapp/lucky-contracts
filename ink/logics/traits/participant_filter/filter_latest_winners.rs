use crate::traits::participant_filter::ParticipantFilterError;
use crate::traits::RAFFLE_MANAGER_ROLE;
use ink::prelude::collections::vec_deque::VecDeque;
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::*;
use openbrush::traits::{AccountId, Storage};
use phat_rollup_anchor_ink::traits::rollup_anchor::RollupAnchor;
use scale::Encode;

const LAST_WINNERS: u32 = ink::selector_id!("LAST_WINNER");

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nb_filtered_winners: u16,
    /// last winners to exclude
    last_winners: VecDeque<AccountId>,
}

#[openbrush::trait_definition]
pub trait FilterLatestWinners: Storage<Data> + access_control::Internal + RollupAnchor {
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn set_nb_winners_filtered(
        &mut self,
        nb_filtered_winners: u16,
    ) -> Result<(), ParticipantFilterError> {
        self.data::<Data>().nb_filtered_winners = nb_filtered_winners;
        Ok(())
    }

    #[ink(message)]
    fn get_nb_winners_filtered(&self) -> u16 {
        self.data::<Data>().nb_filtered_winners
    }

    fn add_winner(&mut self, winner: AccountId) {
        // add the last winner in the back
        self.data::<Data>().last_winners.push_back(winner);
        if self.data::<Data>().last_winners.len() > self.data::<Data>().nb_filtered_winners as usize
        {
            // remove the oldest winner (from the front)
            self.data::<Data>().last_winners.pop_front();
        }
        // save the excluded addresses in the kv store
        let excluded_addresses = Vec::from(self.data::<Data>().last_winners.clone());
        RollupAnchor::set_value(
            self,
            &LAST_WINNERS.encode(),
            Some(&excluded_addresses.encode()),
        );
    }

    #[ink(message)]
    fn get_last_winners(&self) -> Vec<AccountId> {
        Vec::from(self.data::<Data>().last_winners.clone())
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER_ROLE))]
    fn add_address_in_last_winner(
        &mut self,
        winner: AccountId,
    ) -> Result<(), ParticipantFilterError> {
        self.add_winner(winner);
        Ok(())
    }
}
