use crate::traits::participant_filter::ParticipantFilterError;
use ink::prelude::collections::vec_deque::VecDeque;
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::{AccountId, Storage};

pub const PARTICIPANT_FILTER_MANAGER: RoleType = ink::selector_id!("PARTICIPANT_FILTER_MANAGER");

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nb_filtered_winners: u16,
    /// last winners to exclude
    last_winners: VecDeque<AccountId>,
}

#[openbrush::trait_definition]
pub trait FilterLatestWinners: Storage<Data> + access_control::Internal {
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_FILTER_MANAGER))]
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

    fn _add_winner(&mut self, winner: AccountId) {
        // add the last winner in the back
        self.data::<Data>().last_winners.push_back(winner);
        if self.data::<Data>().last_winners.len() > self.data::<Data>().nb_filtered_winners as usize
        {
            // remove the oldest winner (from the front)
            self.data::<Data>().last_winners.pop_front();
        }
    }

    #[ink(message)]
    fn get_last_winners(&self) -> Vec<AccountId> {
        Vec::from(self.data::<Data>().last_winners.clone())
    }

    fn _is_in_last_winners(&self, participant: &AccountId) -> bool {
        self.data::<Data>().last_winners.contains(participant)
    }
}
