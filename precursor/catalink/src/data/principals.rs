use std::{collections::HashMap, sync::LazyLock};

use puckmsg::{ID, Key, KeyID, Principal};

pub static PRINCIPALS: LazyLock<HashMap<&str, Principal>> = LazyLock::new(|| {
    use base64ct::{Base64, Encoding};
    HashMap::from([
        (
            "United States",
            Principal {
                id: ID(1000017249),
                our: Key {
                    id: KeyID(4066866142),
                    secret: Base64::decode_vec("f0xL+f6pjOVyu8DmdOC9GsRDeuBEdowfx6gOosr3efk=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(182081903),
                    secret: Base64::decode_vec("4xNvF0nVH2dbTcU40/vdHvn/+8yui2+fVjVijNtNHB0=")
                        .unwrap(),
                },
            },
        ),
        (
            "United Kingdom",
            Principal {
                id: ID(440025887),
                our: Key {
                    id: KeyID(4208614404),
                    secret: Base64::decode_vec("FQwnSM+WgvGcMWmDpc8/RlDn99UkBPLadAnFspWgmGY=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(3532457197),
                    secret: Base64::decode_vec("s3uaF8/vIoYkqkDZetaeXfzsgr/K1fQPpnhgXdizQ2I=")
                        .unwrap(),
                },
            },
        ),
        (
            "France",
            Principal {
                id: ID(330084672),
                our: Key {
                    id: KeyID(992397266),
                    secret: Base64::decode_vec("+9px+eplIizC7DTdN7A2S5ypqNunYixzNlNe2RVMDms=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(1401929726),
                    secret: Base64::decode_vec("tb+XmdI93oTLG0bAe5KrU2uP/gfPsruylRK6tBwg/w8=")
                        .unwrap(),
                },
            },
        ),
        (
            "China",
            Principal {
                id: ID(860021837),
                our: Key {
                    id: KeyID(3909709118),
                    secret: Base64::decode_vec("CU/4wgIoTCV9fbV7pXDnXb82vwfGk81HBHTKGUg50XI=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(3228425804),
                    secret: Base64::decode_vec("AGAXKLZAKUMHjMc2TcMsnAOAqDkk0pGJQSR9A8y5+Hw=")
                        .unwrap(),
                },
            },
        ),
        (
            "India",
            Principal {
                id: ID(910093479),
                our: Key {
                    id: KeyID(995075093),
                    secret: Base64::decode_vec("Pr4sE0tWNCj7wPiw3cGP8nTGDqwPfADQthBUlM9q9FA=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(1682783621),
                    secret: Base64::decode_vec("9MvrYNkZElokJywsXCkW6IXX1GEduGtRBp3DeL+YG74=")
                        .unwrap(),
                },
            },
        ),
        (
            "Pakistan",
            Principal {
                id: ID(920012415),
                our: Key {
                    id: KeyID(2032870825),
                    secret: Base64::decode_vec("tQtbCs49kbp432lfJEgurQEmqCRFvLGEuD9CNAHF8eg=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(2818687457),
                    secret: Base64::decode_vec("uVXIz0DTx5JjnXpPx5l+iRzWKpp2ipvOv5ccqTJPTBo=")
                        .unwrap(),
                },
            },
        ),
        (
            "Russia",
            Principal {
                id: ID(700085723),
                our: Key {
                    id: KeyID(1610325833),
                    secret: Base64::decode_vec("zKyOEU78AKkMQB+c4GO1XGH+UlK8F73s1jlCDyck4qE=")
                        .unwrap(),
                },
                their: Key {
                    id: KeyID(2426210306),
                    secret: Base64::decode_vec("QwpCG2xUFJEtDNbYDiSl+MiA2BGXgOt3qoAqcNUliSQ=")
                        .unwrap(),
                },
            },
        ),
    ])
});
