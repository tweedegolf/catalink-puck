use core::time::Duration;
use std::time::SystemTime;

use puckmsg::ValidatedMessage;

use crate::data::Contact;

fn contact_usa() -> Contact {
    Contact {
        name: "United States".to_string(),
    }
}

fn contact_china() -> Contact {
    Contact {
        name: "China".to_string(),
    }
}

fn contact_russia() -> Contact {
    Contact {
        name: "Russia".to_string(),
    }
}

pub fn contacts() -> Vec<Contact> {
    vec![
        contact_usa(),
        contact_china(),
        contact_russia(),
        Contact {
            name: "France".to_string(),
        },
        Contact {
            name: "United Kingdom".to_string(),
        },
        Contact {
            name: "India".to_string(),
        },
        Contact {
            name: "Jeremy".to_string(),
        },
        Contact {
            name: "Susan".to_string(),
        },
        Contact {
            name: "Susan2".to_string(),
        },
        Contact {
            name: "Jeremy3".to_string(),
        },
    ]
}

fn lipsum() -> Vec<u8> {
    const LIPSUM: &str = "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.";
    Vec::from(LIPSUM.as_bytes())
}

pub fn messages() -> Vec<ValidatedMessage> {
    let protos = [
        (contact_usa(), vec![contact_russia(), contact_china()]),
        (contact_usa(), vec![contact_russia(), contact_china()]),
        (contact_usa(), vec![contact_russia(), contact_china()]),
        (contact_russia(), vec![contact_china(), contact_usa()]),
        (contact_usa(), vec![contact_russia(), contact_china()]),
    ];

    let now = SystemTime::now();

    protos
        .into_iter()
        .enumerate()
        .map(|(i, (sender, recipients))| ValidatedMessage {
            sender: sender.name,
            recipients: recipients
                .into_iter()
                .map(|recipient| recipient.name)
                .collect(),
            body: lipsum(),
            time: now.checked_add(Duration::from_hours(i as u64)).unwrap(),
        })
        .collect()
}
