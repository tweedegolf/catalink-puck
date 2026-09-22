pub type Rng = rand::rngs::ThreadRng;

use std::collections::HashMap;
use std::sync::LazyLock;

use puckmsg::{ID, Key, KeyID, Principal};

pub static PRINCIPALS: LazyLock<HashMap<&str, Principal>> = LazyLock::new(|| {
    use base64ct::{Base64, Encoding};
    HashMap::from([
        (
            "gopuck",
            Principal {
                id: ID(1412638488),
                their: Key {
                    id: KeyID(1667057689),
                    secret: Base64::decode_vec("Ux7sv5J5zOqA4TSdbkFvqX11fl4FAhLhjmfTQ9pN3q0=")
                        .unwrap(),
                },
                our: Key {
                    id: KeyID(64312123),
                    secret: Base64::decode_vec("Sha0OcAPzzUSyN9iuLN8XDponCBD8A/KBp6H2LsvAss=")
                        .unwrap(),
                },
            },
        ),
        (
            "eric",
            Principal {
                id: ID(1446134797),
                our: Key {
                    id: KeyID(1667057689),
                    secret: Base64::decode_vec("Ux7sv5J5zOqA4TSdbkFvqX11fl4FAhLhjmfTQ9pN3q0=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(64312123),
                    secret: Base64::decode_vec("Sha0OcAPzzUSyN9iuLN8XDponCBD8A/KBp6H2LsvAss=")
                        .unwrap(),
                },
            },
        ),
    ])
});
