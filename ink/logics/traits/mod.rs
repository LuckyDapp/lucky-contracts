use openbrush::contracts::access_control::RoleType;

pub const RAFFLE_MANAGER_ROLE: RoleType = ink::selector_id!("RAFFLE_MANAGER");

pub mod participant_filter;
pub mod raffle;
pub mod reward;
