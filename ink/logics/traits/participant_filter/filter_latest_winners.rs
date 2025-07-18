use crate::traits::RAFFLE_MANAGER_ROLE;
use crate::traits::error::RaffleError;
use ink::prelude::collections::vec_deque::VecDeque;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use inkv5_client_lib::traits::access_control::{BaseAccessControl};
use inkv5_client_lib::traits::kv_store::KvStore;
use ink::env::DefaultEnvironment;
use ink::scale::{Encode};


const LAST_WINNERS: u32 = ink::selector_id!("LAST_WINNER");

#[derive(Default, Debug)]
#[ink::storage_item]
pub struct FilterLatestWinnersData {
    nb_filtered_winners: u16,
    /// last winners to exclude
    last_winners: VecDeque<AccountId>,
}

#[ink::trait_definition]
pub trait FilterLatestWinners {
    #[ink(message)]
    fn set_nb_winners_filtered(
        &mut self,
        nb_filtered_winners: u16,
    ) -> Result<(), RaffleError>;

    #[ink(message)]
    fn get_nb_winners_filtered(&self) -> u16;

    #[ink(message)]
    fn get_last_winners(&self) -> Vec<AccountId> ;

    #[ink(message)]
    fn add_address_in_last_winner(
        &mut self,
        winner: AccountId,
    ) -> Result<(), RaffleError> ;
}


pub trait FilterLatestWinnersStorage {
    fn get_storage(&self) -> &FilterLatestWinnersData;
    fn get_mut_storage(&mut self) -> &mut FilterLatestWinnersData;
}


pub trait BaseFilterLatestWinners: FilterLatestWinnersStorage + KvStore + BaseAccessControl {
    
    fn inner_set_nb_winners_filtered(
        &mut self,
        nb_filtered_winners: u16,
    ) -> Result<(), RaffleError> {

        let caller = ::ink::env::caller::<DefaultEnvironment>();
        self.inner_check_role(RAFFLE_MANAGER_ROLE, caller)?;

        FilterLatestWinnersStorage::get_mut_storage(self).nb_filtered_winners = nb_filtered_winners;
        Ok(())
    }
    
    fn inner_get_nb_winners_filtered(&self) -> u16 {
        FilterLatestWinnersStorage::get_storage(self).nb_filtered_winners
    }

    fn add_winner(&mut self, winner: AccountId) {
        // add the last winner in the back
        FilterLatestWinnersStorage::get_mut_storage(self).last_winners.push_back(winner);
        if FilterLatestWinnersStorage::get_storage(self).last_winners.len() > FilterLatestWinnersStorage::get_storage(self).nb_filtered_winners as usize
        {
            // remove the oldest winner (from the front)
            FilterLatestWinnersStorage::get_mut_storage(self).last_winners.pop_front();
        }
        // save the excluded addresses in the kv store
        let excluded_addresses = Vec::from(FilterLatestWinnersStorage::get_storage(self).last_winners.clone());
        KvStore::inner_set_value(
            self,
            &LAST_WINNERS.encode(),
            Some(&excluded_addresses.encode()),
        );
    }

    fn inner_get_last_winners(&self) -> Vec<AccountId> {
        Vec::from(FilterLatestWinnersStorage::get_storage(self).last_winners.clone())
    }

    fn inner_add_address_in_last_winner(
        &mut self,
        winner: AccountId,
    ) -> Result<(), RaffleError> {
        
        let caller = ::ink::env::caller::<DefaultEnvironment>();
        self.inner_check_role(RAFFLE_MANAGER_ROLE, caller)?;
        
        self.add_winner(winner);
        Ok(())
    }
}