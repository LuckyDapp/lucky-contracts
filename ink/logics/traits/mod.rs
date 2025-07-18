
pub const RAFFLE_MANAGER_ROLE: u32 = ink::selector_id!("RAFFLE_MANAGER");

pub type Balance = u128;

pub mod error;
pub mod participant_filter;
pub mod raffle;
pub mod reward;