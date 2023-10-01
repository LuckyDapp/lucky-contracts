#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[openbrush::implementation(AccessControl)]
#[openbrush::contract]
pub mod random_generator {
    use openbrush::contracts::access_control::{access_control, *};
    use openbrush::traits::Storage;

    use lucky::traits::{random_generator::*, *};

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        random_generator: random_generator::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl RandomGenerator for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.random_generator = random_generator::Data::default();
            let caller = instance.env().caller();
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            AccessControl::grant_role(&mut instance, RANDOM_GENERATOR_CONSUMER, Some(caller))
                .expect("Should grant the role RANDOM_GENERATOR_CONSUMER");
            AccessControl::grant_role(&mut instance, RANDOM_GENERATOR_MANAGER, Some(caller))
                .expect("Should grant the role RANDOM_GENERATOR_MANAGER");
            instance
        }
    }

    mod tests {

        use super::*;

        #[ink::test]
        fn test_get_pseudo_random() {
            let mut contract = Contract::new();
            for max_value in 0..=100 {
                for min_value in 0..=max_value {
                    let result = contract.get_random_number(min_value, max_value).unwrap();
                    assert!(result >= min_value);
                    assert!(result <= max_value);
                }
            }
        }
    }
}
