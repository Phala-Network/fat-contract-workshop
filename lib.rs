#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod fat_sample {
    use super::pink;
    use alloc::{string::String, vec::Vec};
    use pink::{http_get, PinkEnvironment};
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct FatSample {}

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidUrl,
        RequestFailed,
        NoClaimFound,
        InvalidAddressLength,
        InvalidAddress,
    }

    impl FatSample {
        #[ink(constructor)]
        pub fn default() -> Self {
            // TODO-1. generate a Sr25519 key pair
            // TODO-2. (optionally) reveal the pubkey
            // TODO-3. save sender as admin
            Self {}
        }

        // TODO-4. admin set_poap_links(links: Vec<Vec<u8>>)
        // TODO-5. redeem(attestation)
        // TODO-6. query my_link()

        #[ink(message)]
        pub fn query_example(&self) -> (u16, Vec<u8>) {
            let resposne = http_get!("https://example.com");
            (resposne.status_code, resposne.body)
        }

        #[ink(message)]
        pub fn attest_gist(&self, url: Vec<u8>) -> Result<SignedAttestation, Error> {
            // Verify the URL
            let gist_url = parse_gist_url(&url)?;
            let url = String::from_utf8(url).or(Err(Error::InvalidUrl))?;
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
            Ok(attestation.into_signed(b"TODO: privkey"))
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    struct GistUrl {
        username: Vec<u8>,
        gist_id: Vec<u8>,
        filename: Vec<u8>,
    }

    /// Parses a Github Gist url.
    ///
    /// - Returns a parsed [GistUrl] struct if the input is a valid url;
    /// - Otherwise returns an [Error].
    fn parse_gist_url(url: &[u8]) -> Result<GistUrl, Error> {
        let path = url
            .strip_prefix(b"https://gist.githubusercontent.com/")
            .ok_or(Error::InvalidUrl)?;
        let components: Vec<_> = path.split(|c| *c == b'/').collect();
        if components.len() < 5 {
            return Err(Error::InvalidUrl);
        }
        Ok(GistUrl {
            username: components[0].to_vec(),
            gist_id: components[1].to_vec(),
            filename: components[4].to_vec(),
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
        username: Vec<u8>,
        account_id: AccountId,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SignedAttestation {
        attestation: Attestation,
        signature: Vec<u8>,
    }

    impl Attestation {
        fn into_signed(self, key: &[u8]) -> SignedAttestation {
            let encoded = Encode::encode(&self);
            // let signature = sign(encoded, key);
            SignedAttestation {
                attestation: self,
                signature: Vec::new(), // TODO
            }
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

        #[ink::test]
        fn can_parse_gist_url() {
            let result = parse_gist_url(b"https://gist.githubusercontent.com/h4x3rotab/0cabeb528bdaf30e4cf741e26b714e04/raw/620f958fb92baba585a77c1854d68dc986803b4e/test%2520gist");
            assert_eq!(
                result,
                Ok(GistUrl {
                    username: b"h4x3rotab".to_vec(),
                    gist_id: b"0cabeb528bdaf30e4cf741e26b714e04".to_vec(),
                    filename: b"test%2520gist".to_vec(),
                })
            );
            let err = parse_gist_url(b"http://example.com");
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
    }
}
