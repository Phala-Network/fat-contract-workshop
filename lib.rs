#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod fat_sample {
    use super::pink;
    use pink::{PinkEnvironment, http_get};
    use alloc::vec::Vec;

    #[ink(storage)]
    pub struct FatSample {}

    impl FatSample {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn query_example(&self) -> (u16, Vec<u8>) {
            let resposne = http_get!("https://example.com");
            (resposne.status_code, resposne.body)
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
        }
    }
}
