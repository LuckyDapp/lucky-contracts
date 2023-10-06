use crate::traits::random_generator::RandomGeneratorError::*;
use ink::env::hash::{HashOutput, Keccak256};
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, AccessControlError, RoleType};
use openbrush::traits::Storage;

#[openbrush::wrapper]
pub type RandomGeneratorRef = dyn RandomGenerator;

pub const RANDOM_GENERATOR_CONSUMER: RoleType = ink::selector_id!("RANDOM_GENERATOR_CONSUMER");
pub const RANDOM_GENERATOR_MANAGER: RoleType = ink::selector_id!("RANDOM_GENERATOR_MANAGER");

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    salt: u64,
}

#[openbrush::trait_definition]
pub trait RandomGenerator: Storage<Data> + access_control::Internal {
    /// generate a random number between min and max values.
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_CONSUMER))]
    fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomGeneratorError> {
        let seed = Self::env().block_timestamp();
        let salt = self.data::<Data>().salt;
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&seed.to_be_bytes());
        input.extend_from_slice(&salt.to_be_bytes());
        let mut output = <Keccak256 as HashOutput>::Type::default();
        ink::env::hash_bytes::<Keccak256>(&input, &mut output);
        self.data::<Data>().salt = salt + 1;

        let a = output[0] as u128;

        //(a  as u32) * (max - min) / (u32::MAX) + min
        let b = max.checked_sub(min).ok_or(SubOverFlow)?;
        let c = a.checked_mul(b).ok_or(MulOverFlow)?;
        let d = c.checked_div(u8::MAX as u128).ok_or(DivByZero)?;
        let e = d.checked_add(min).ok_or(AddOverFlow)?;

        ink::env::debug_println!("random {}", e);

        Ok(e)
    }

    /// get the current salt used for randomness
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_MANAGER))]
    fn get_salt(&mut self) -> Result<u64, RandomGeneratorError> {
        Ok(self.data::<Data>().salt)
    }

    /// Set the current salt used for randomness
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_MANAGER))]
    fn set_salt(&mut self, salt: u64) -> Result<(), RandomGeneratorError> {
        self.data::<Data>().salt = salt;
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomGeneratorError {
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    SubOverFlow,
    MissingAddress,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RandomGeneratorError
impl From<AccessControlError> for RandomGeneratorError {
    fn from(error: AccessControlError) -> Self {
        RandomGeneratorError::AccessControlError(error)
    }
}
