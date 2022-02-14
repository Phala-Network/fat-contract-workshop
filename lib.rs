#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod fat_sample {
    use super::pink;
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };
    use ink_storage::{lazy::Mapping, traits::SpreadAllocate};
    use pink::{
        chain_extension::SigType, derive_sr25519_pair, http_get, sign, verify, PinkEnvironment,
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct FatSample {
        admin: AccountId,
        attestation_privkey: Vec<u8>,
        attestation_pubkey: Vec<u8>,
        poap_code: Vec<String>,

        /// Map from the account to the redemption index
        ///
        /// Thus the POAP code should be `poap_code[index]`.
        redeem_by_account: Mapping<AccountId, u32>,
        /// The number of total redeemed code.
        total_redeemed: u32,
        /// Map from verified accounts to usernames
        username_by_account: Mapping<AccountId, String>,
        /// Map from verified usernames to accounts
        account_by_username: Mapping<String, AccountId>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidUrl,
        RequestFailed,
        NoClaimFound,
        InvalidAddressLength,
        InvalidAddress,
        NoPermission,
        InvalidSignature,
        UsernameAlreadyInUse,
        AccountAlreadyInUse,
    }

    impl FatSample {
        #[ink(constructor)]
        pub fn default() -> Self {
            // Generate a Sr25519 key pair
            let (privkey, pubkey) = derive_sr25519_pair!(b"gist-attestation-key");
            // Save sender as the contract admin
            let admin = Self::env().caller();

            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::codegen::initialize_contract(|contract: &mut Self| {
                contract.admin = admin;
                contract.attestation_privkey = privkey;
                contract.attestation_pubkey = pubkey;
                contract.total_redeemed = 0u32;
            })
        }

        /// Sets the POAP redemption code. (callable, admin-only)
        ///
        /// The admin must set enough POAP code while setting up the contract. The code can be
        /// overrode at any time.
        #[ink(message)]
        pub fn admin_set_poap_code(&mut self, code: Vec<String>) -> Result<(), Error> {
            // The caller must be the admin
            let caller = self.env().caller();
            if caller != self.admin {
                return Err(Error::NoPermission);
            }
            // Update the code
            self.poap_code = code;
            Ok(())
        }

        /// Redeems a POAP with a signed `attestation`. (callable)
        ///
        /// The attestation must be created by [attest_gist] function. After the verification of
        /// the attestation, the the sender account will the linked to a Github username. Then a
        /// POAP redemption code will be allocated to the sender.
        ///
        /// Each blockchain account and github account can only be linked once.
        #[ink(message)]
        pub fn redeem(&mut self, signed: SignedAttestation) -> Result<(), Error> {
            // Verify the attestation
            let attestation = self.verify_attestation(signed)?;
            // The caller must be the attested account
            if attestation.account_id != self.env().caller() {
                return Err(Error::NoPermission);
            }
            let username = attestation.username;
            let account = attestation.account_id;
            // The username and the account mustn't be linked
            if self.username_by_account.get(&account).is_some() {
                return Err(Error::UsernameAlreadyInUse);
            }
            if self.account_by_username.get(&username).is_some() {
                return Err(Error::AccountAlreadyInUse);
            }
            // Link the accounts, and prevent double redemptions
            self.username_by_account.insert(&account, &username);
            self.account_by_username.insert(&username, &account);
            self.redeem_by_account
                .insert(&account, &self.total_redeemed);
            self.total_redeemed += 1;
            Ok(())
        }

        /// Returns my POAP redemption code / link if it exists. (View function)
        ///
        /// - If the account doesn't have any redemption code allocated, it returns `None`;
        /// - If the account has the code allocated but the contract doesn't have sufficient code
        ///    in `poap_code`, it returns `None` as well;
        /// - Otherwise it returns the code.
        #[ink(message)]
        pub fn my_poap(&self) -> Option<String> {
            let caller = self.env().caller();
            let idx = match self.redeem_by_account.get(&caller) {
                Some(idx) => idx,
                None => return None,
            };
            self.poap_code.get(idx as usize).cloned()
        }

        #[ink(message)]
        pub fn query_example(&self) -> (u16, Vec<u8>) {
            let resposne = http_get!("https://example.com");
            (resposne.status_code, resposne.body)
        }

        /// Attests a Github Gist by the raw file url. (Query only)
        ///
        /// It sends a HTTPS request to the url and extract an address from the claim ("This gist
        /// is owned by address: 0x..."). Once the claim is verified, it returns a signed
        /// attestation with the pair `(github_username, account_id)`.
        #[ink(message)]
        pub fn attest_gist(&self, url: String) -> Result<SignedAttestation, Error> {
            // Verify the URL
            let gist_url = parse_gist_url(&url)?;
            // Fetch the gist content
            let resposne = http_get!(url);
            if resposne.status_code != 200 {
                return Err(Error::RequestFailed);
            }
            let body = resposne.body;
            // Verify the claim and extract the account id
            let account_id = extract_claim(&body)?;
            let attestation = Attestation {
                username: gist_url.username,
                account_id,
            };
            let result = self.sign_attestation(attestation);
            Ok(result)
        }

        /// Signs the `attestation` with the attestation key pair.
        pub fn sign_attestation(&self, attestation: Attestation) -> SignedAttestation {
            let encoded = Encode::encode(&attestation);
            let signature = sign!(&encoded, &self.attestation_privkey, SigType::Sr25519);
            SignedAttestation {
                attestation,
                signature,
            }
        }

        /// Verifies the signed attestation and return the inner data.
        pub fn verify_attestation(&self, signed: SignedAttestation) -> Result<Attestation, Error> {
            let encoded = Encode::encode(&signed.attestation);
            if !verify!(
                &encoded,
                &self.attestation_pubkey,
                &signed.signature,
                SigType::Sr25519
            ) {
                return Err(Error::InvalidSignature);
            }
            Ok(signed.attestation)
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    struct GistUrl {
        username: String,
        gist_id: String,
        filename: String,
    }

    /// Parses a Github Gist url.
    ///
    /// - Returns a parsed [GistUrl] struct if the input is a valid url;
    /// - Otherwise returns an [Error].
    fn parse_gist_url(url: &str) -> Result<GistUrl, Error> {
        let path = url
            .strip_prefix("https://gist.githubusercontent.com/")
            .ok_or(Error::InvalidUrl)?;
        let components: Vec<_> = path.split('/').collect();
        if components.len() < 5 {
            return Err(Error::InvalidUrl);
        }
        Ok(GistUrl {
            username: components[0].to_string(),
            gist_id: components[1].to_string(),
            filename: components[4].to_string(),
        })
    }

    const CLAIM_PREFIX: &str = "This gist is owned by address: 0x";
    const ADDRESS_LEN: usize = 64;

    /// Extracts the ownerhip of the gist from a claim in the gist body.
    ///
    /// A valid claim must have the statement "This gist is owned by address: 0x..." in `body`. The
    /// address must be the 256 bits public key of the Substrate account in hex.
    ///
    /// - Returns a 256-bit `AccountId` representing the owner account if the claim is valid;
    /// - otherwise returns an [Error].
    fn extract_claim(body: &[u8]) -> Result<AccountId, Error> {
        let body = String::from_utf8_lossy(body);
        let pos = body.find(CLAIM_PREFIX).ok_or(Error::NoClaimFound)?;
        let addr: String = body
            .chars()
            .skip(pos)
            .skip(CLAIM_PREFIX.len())
            .take(ADDRESS_LEN)
            .collect();
        let addr = addr.as_bytes();
        let account_id = decode_accountid_256(addr)?;
        Ok(account_id)
    }

    /// Decodes a hex string as an 256-bit AccountId32
    fn decode_accountid_256(addr: &[u8]) -> Result<AccountId, Error> {
        use hex::FromHex;
        if addr.len() != ADDRESS_LEN {
            return Err(Error::InvalidAddressLength);
        }
        let bytes = <[u8; 32]>::from_hex(addr).or(Err(Error::InvalidAddress))?;
        Ok(AccountId::from(bytes))
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Attestation {
        username: String,
        account_id: AccountId,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SignedAttestation {
        attestation: Attestation,
        signature: Vec<u8>,
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

        #[ink::test]
        fn can_parse_gist_url() {
            let result = parse_gist_url("https://gist.githubusercontent.com/h4x3rotab/0cabeb528bdaf30e4cf741e26b714e04/raw/620f958fb92baba585a77c1854d68dc986803b4e/test%2520gist");
            assert_eq!(
                result,
                Ok(GistUrl {
                    username: "h4x3rotab".to_string(),
                    gist_id: "0cabeb528bdaf30e4cf741e26b714e04".to_string(),
                    filename: "test%2520gist".to_string(),
                })
            );
            let err = parse_gist_url("http://example.com");
            assert_eq!(err, Err(Error::InvalidUrl));
        }

        #[ink::test]
        fn can_decode_claim() {
            use hex::FromHex;

            let ok = extract_claim(b"...This gist is owned by address: 0x0123456789012345678901234567890123456789012345678901234567890123...");
            assert_eq!(
                ok,
                Ok(AccountId::from(
                    <[u8; 32]>::from_hex(
                        "0123456789012345678901234567890123456789012345678901234567890123"
                    )
                    .unwrap()
                ))
            );
            // Bad cases
            assert_eq!(
                extract_claim(b"This gist is owned by"),
                Err(Error::NoClaimFound),
            );
            assert_eq!(
                extract_claim(b"This gist is owned by address: 0xAB"),
                Err(Error::InvalidAddressLength),
            );
            assert_eq!(
                extract_claim(b"This gist is owned by address: 0xXX23456789012345678901234567890123456789012345678901234567890123"),
                Err(Error::InvalidAddress),
            );
        }

        #[ink::test]
        fn end_to_end() {
            use pink_extension::chain_extension::{test::*, HttpResponse};

            // Mock derive key call (a pregenerated key pair)
            ink_env::test::register_chain_extension(MockDeriveSr25519Pair::new(|_| {
                (
                    hex::decode("78003ee90ff2544789399de83c60fa50b3b24ca86c7512d0680f64119207c80ab240b41344968b3e3a71a02c0e8b454658e00e9310f443935ecadbdd1674c683").unwrap(),
                    hex::decode("ce786c340288b79a951c68f87da821d6c69abd1899dff695bda95e03f9c0b012").unwrap()
                )
            }));
            ink_env::test::register_chain_extension(MockSign::new(|_| b"mock-signature".to_vec()));
            ink_env::test::register_chain_extension(MockVerify::new(|_| true));

            // Test accounts
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
            // Construct a contract (deployed by `accounts.alice` by default)
            let mut contract = FatSample::default();
            assert_eq!(contract.admin, accounts.alice);
            // Admin (alice) can set POAP
            assert!(contract
                .admin_set_poap_code(vec!["code1".to_string(), "code2".to_string(),])
                .is_ok());
            // Generate an attestation
            //
            // Mock a http request first (the 256 bits account id is the pubkey of Alice)
            ink_env::test::register_chain_extension(MockHttpRequest::new(|_| {
                HttpResponse::ok(b"This gist is owned by address: 0x0101010101010101010101010101010101010101010101010101010101010101".to_vec())
            }));
            let result = contract.attest_gist("https://gist.githubusercontent.com/h4x3rotab/0cabeb528bdaf30e4cf741e26b714e04/raw/620f958fb92baba585a77c1854d68dc986803b4e/test%2520gist".to_string());
            assert!(result.is_ok());
            let attestation = result.unwrap();
            assert_eq!(attestation.attestation.username, "h4x3rotab");
            assert_eq!(attestation.attestation.account_id, accounts.alice);
            // Redeem
            assert!(contract.redeem(attestation).is_ok());
            assert_eq!(contract.total_redeemed, 1);
            assert_eq!(
                contract.account_by_username.get("h4x3rotab".to_string()),
                Some(accounts.alice)
            );
            assert_eq!(
                contract.username_by_account.get(&accounts.alice),
                Some("h4x3rotab".to_string())
            );
            assert_eq!(contract.redeem_by_account.get(accounts.alice), Some(0));
        }
    }
}
