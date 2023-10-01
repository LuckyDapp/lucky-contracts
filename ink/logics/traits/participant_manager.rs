use ink::prelude::vec::Vec;
use ink::storage::Lazy;
use openbrush::contracts::access_control::{access_control, AccessControlError, RoleType};
use openbrush::traits::{AccountId, Balance, Storage};

pub const PARTICIPANT_MANAGER: RoleType = ink::selector_id!("PARTICIPANT_MANAGER");
pub const MAX_PART_BY_VEC: usize = 150;
pub const MAX_PART: usize = MAX_PART_BY_VEC * 6;

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Participant {
    pub account: AccountId,
    pub value: Balance,
}

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nb_participants: u16,
    /// participants
    /// to not reach max capacity size, we will split the participants in many vectors (max 300 participants by vector)
    #[lazy]
    participants_1: Vec<Participant>,
    total_value_1: Balance,
    #[lazy]
    participants_2: Vec<Participant>,
    total_value_2: Balance,
    #[lazy]
    participants_3: Vec<Participant>,
    total_value_3: Balance,
    #[lazy]
    participants_4: Vec<Participant>,
    total_value_4: Balance,
    #[lazy]
    participants_5: Vec<Participant>,
    total_value_5: Balance,
    #[lazy]
    participants_6: Vec<Participant>,
    total_value_6: Balance,
}

fn push_participants(
    index: usize,
    src: &Vec<(AccountId, Balance)>,
    dest: &mut Vec<Participant>,
) -> (usize, Balance) {
    let mut total_value = Balance::default();

    let remaining_len = MAX_PART_BY_VEC - dest.len();
    let mut end_index = index + remaining_len;
    if end_index >= src.len() {
        end_index = src.len();
    }

    for (account, value) in src[index..end_index].iter() {
        dest.push(Participant {
            account: *account,
            value: *value,
        });
        total_value += *value;
    }

    (end_index - index, total_value)
}

/// Iterate on the participants, sum the values,
/// and return the participant if the sum is superior to the given weight
fn select_winner_matching_value(
    participants: &Vec<Participant>,
    selected_value: Balance,
) -> Option<AccountId> {
    let mut total_value = 0;
    for participant in participants {
        total_value += participant.value;
        if total_value >= selected_value {
            return Some(participant.account);
        }
    }
    None
}

#[openbrush::trait_definition]
pub trait ParticipantManager: Storage<Data> + access_control::Internal {
    #[ink(message)]
    fn get_nb_participants(&self) -> u16 {
        self.data::<Data>().nb_participants
    }

    #[ink(message)]
    fn get_total_value(&self) -> Balance {
        self.data::<Data>().total_value_1
            + self.data::<Data>().total_value_2
            + self.data::<Data>().total_value_3
            + self.data::<Data>().total_value_4
            + self.data::<Data>().total_value_5
            + self.data::<Data>().total_value_6
    }

    #[ink(message)]
    fn get_participant(&self, value: Balance) -> Option<AccountId> {
        let mut to_value = self.data::<Data>().total_value_1;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_1.get_or_default(),
                value,
            );
        }
        let mut from_value = to_value;
        to_value += self.data::<Data>().total_value_2;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_2.get_or_default(),
                value - from_value,
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_3;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_3.get_or_default(),
                value - from_value,
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_4;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_4.get_or_default(),
                value - from_value,
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_5;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_5.get_or_default(),
                value - from_value,
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_6;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_6.get_or_default(),
                value - from_value,
            );
        }
        None
    }

    #[ink(message)]
    fn get_participants(&self, page: u8) -> Result<Vec<Participant>, ParticipantManagerError> {
        let participants;
        if page == 0 {
            participants = Vec::new();
        } else if page == 1 {
            participants = self.data::<Data>().participants_1.get_or_default();
        } else if page == 2 {
            participants = self.data::<Data>().participants_2.get_or_default();
        } else if page == 3 {
            participants = self.data::<Data>().participants_3.get_or_default();
        } else if page == 4 {
            participants = self.data::<Data>().participants_4.get_or_default();
        } else if page == 5 {
            participants = self.data::<Data>().participants_5.get_or_default();
        } else if page == 6 {
            participants = self.data::<Data>().participants_6.get_or_default();
        } else {
            return Err(ParticipantManagerError::PageNotFound);
        }

        Ok(participants)
    }

    /// add participants in the raffle
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_MANAGER))]
    fn add_participants(
        &mut self,
        participants: Vec<(AccountId, Balance)>,
    ) -> Result<(), ParticipantManagerError> {
        let mut nb_participants = self.data::<Data>().nb_participants as usize;
        let mut index = 0;

        while index < participants.len() {
            let inserted_participants;

            if nb_participants < MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_1.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_1 += total_value;
                self.data::<Data>().participants_1.set(&p);
            } else if nb_participants < 2 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_2.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_2 += total_value;
                self.data::<Data>().participants_2.set(&p);
            } else if nb_participants < 3 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_3.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_3 += total_value;
                self.data::<Data>().participants_3.set(&p);
            } else if nb_participants < 4 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_4.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_4 += total_value;
                self.data::<Data>().participants_4.set(&p);
            } else if nb_participants < 5 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_5.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_5 += total_value;
                self.data::<Data>().participants_5.set(&p);
            } else if nb_participants < 6 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_6.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_6 += total_value;
                self.data::<Data>().participants_6.set(&p);
            } else {
                return Err(ParticipantManagerError::MaxSizeExceeded);
            }
            nb_participants = nb_participants + inserted_participants;
            index = index + inserted_participants;
        }
        self.data::<Data>().nb_participants = nb_participants as u16;
        Ok(())
    }

    /// Clear the data (participants and rewards)
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_MANAGER))]
    fn clear_data(&mut self) -> Result<(), ParticipantManagerError> {
        let nb_participants = self.data::<Data>().nb_participants as usize;

        if nb_participants > 0 {
            let mut p = self.data::<Data>().participants_1.get_or_default();
            p.clear();
            self.data::<Data>().participants_1.set(&p);
        }
        if nb_participants > MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_2.get_or_default();
            p.clear();
            self.data::<Data>().participants_2.set(&p);
        }
        if nb_participants > 2 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_3.get_or_default();
            p.clear();
            self.data::<Data>().participants_3.set(&p);
        }
        if nb_participants > 3 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_4.get_or_default();
            p.clear();
            self.data::<Data>().participants_4.set(&p);
        }
        if nb_participants > 4 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_5.get_or_default();
            p.clear();
            self.data::<Data>().participants_5.set(&p);
        }
        if nb_participants > 5 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_6.get_or_default();
            p.clear();
            self.data::<Data>().participants_6.set(&p);
        }

        self.data::<Data>().nb_participants = 0;
        self.data::<Data>().total_value_1 = 0;
        self.data::<Data>().total_value_2 = 0;
        self.data::<Data>().total_value_3 = 0;
        self.data::<Data>().total_value_4 = 0;
        self.data::<Data>().total_value_5 = 0;
        self.data::<Data>().total_value_6 = 0;

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ParticipantManagerError {
    MaxSizeExceeded,
    PageNotFound,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantManagerError
impl From<AccessControlError> for ParticipantManagerError {
    fn from(error: AccessControlError) -> Self {
        ParticipantManagerError::AccessControlError(error)
    }
}
