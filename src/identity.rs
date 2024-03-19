use std::collections::BTreeMap;
use dpp::dashcore::secp256k1::rand;
//use dpp::dashcore::secp256k1::rand::rngs::StdRng;
use dpp::dashcore::secp256k1::rand::SeedableRng;
use dpp::identity::identity_public_key::v0::IdentityPublicKeyV0;
use dpp::identity::identity::Identity;
use dpp::identity::identity_public_key::IdentityPublicKey;
use dpp::identity::v0::IdentityV0;
use dpp::identity::identity_public_key::contract_bounds::ContractBounds;
use dpp::identity::identity_public_key::{KeyID, TimestampMillis};
use dpp::identity::identity_public_key::accessors::v0::IdentityPublicKeyGettersV0;
use dpp::identity::KeyType::ECDSA_SECP256K1;
use dpp::identity::Purpose::AUTHENTICATION;
use dpp::identity::SecurityLevel::MASTER;
use platform_value::IdentifierBytes32;
use platform_value::types::identifier::Identifier;
use platform_version::version::LATEST_PLATFORM_VERSION;

#[ferment_macro::export]
pub fn create_basic_identity(id: [u8; 32]) -> Identity {
    Identity::V0(IdentityV0 {
        id: Identifier(IdentifierBytes32(id)),
        revision: 0,
        balance: 0,
        public_keys: BTreeMap::new(),
    })
}
#[ferment_macro::export]
pub fn get_identity2(identifier: Identifier) -> Identity {
    let id = Identifier::from_bytes(&identifier.as_slice()).expect("parse identity id");

    let mut keys: BTreeMap<KeyID, IdentityPublicKey> = BTreeMap::new();
    let mut rng = rand::rngs::StdRng::from_entropy();

    let key: IdentityPublicKey = IdentityPublicKey::V0(
        IdentityPublicKeyV0::random_ecdsa_master_authentication_key_with_rng(
            1,
            &mut rng,
            LATEST_PLATFORM_VERSION,
        )
            .expect("expected a random key")
            .0
    );

    let key2: IdentityPublicKey = IdentityPublicKey::V0(
        IdentityPublicKeyV0::random_ecdsa_master_authentication_key_with_rng(
            1,
            &mut rng,
            LATEST_PLATFORM_VERSION,
        )
            .expect("expected a random key")
            .0
    );

    keys.insert(1, key);
    keys.insert(2, key2);

    let identity = IdentityV0 {
        id: id,
        public_keys: keys,
        balance: 2,
        revision: 1,
    };
    Identity::V0(identity)
}

#[ferment_macro::export]
pub fn get_identity_contract_bounds(identifier: Identifier, contract_identifier: Option<Identifier>) -> Identity {
    let id = Identifier::from_bytes(&identifier.as_slice()).expect("parse identity id");

    let mut keys: BTreeMap<KeyID, IdentityPublicKey> = BTreeMap::new();
    let mut rng = rand::rngs::StdRng::from_entropy();

    let ipk1 = IdentityPublicKeyV0::random_ecdsa_master_authentication_key_with_rng(
        1,
        &mut rng,
        LATEST_PLATFORM_VERSION,
    ).expect("expected a random key").0;
    let key = IdentityPublicKey::V0(
        IdentityPublicKeyV0 {
            id: ipk1.id(),
            purpose: AUTHENTICATION,
            security_level: MASTER,
            contract_bounds: match contract_identifier {
                Some(id) => Some (ContractBounds::SingleContract { id: id.clone() }),
                None => None
            },
            key_type: ECDSA_SECP256K1,
            read_only: false,
            data: ipk1.data().clone(),
            disabled_at: Some(1)
        }
    );

    let ipk2 = IdentityPublicKeyV0::random_ecdsa_master_authentication_key_with_rng(
        1,
        &mut rng,
        LATEST_PLATFORM_VERSION,
    ).expect("expected a random key").0;
    let key2 = IdentityPublicKey::V0(
        IdentityPublicKeyV0 {
            id: ipk2.id(),
            purpose: AUTHENTICATION,
            security_level: MASTER,
            contract_bounds: match contract_identifier {
                Some(id) => Some (ContractBounds::SingleContract { id: id.clone() }),
                None => None
            },
            key_type: ECDSA_SECP256K1,
            read_only: ipk2.read_only(),
            data: ipk2.data().clone(),
            disabled_at: Some(1)
        }
    );

    keys.insert(1, key);
    keys.insert(2, key2);

    let identity = IdentityV0 {
        id: id,
        public_keys: keys,
        balance: 2,
        revision: 1,
    };
    Identity::V0(identity)
}