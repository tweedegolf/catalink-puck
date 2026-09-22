use chrono::Utc;
use core::time::Duration;

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

fn contact_france() -> Contact {
    Contact {
        name: "France".to_string(),
    }
}

fn contact_uk() -> Contact {
    Contact {
        name: "United Kingdom".to_string(),
    }
}

fn contact_india() -> Contact {
    Contact {
        name: "India".to_string(),
    }
}

pub fn contacts() -> Vec<Contact> {
    vec![
        contact_china(),
        contact_france(),
        contact_india(),
        contact_russia(),
        contact_usa(),
    ]
}

pub fn our_contact() -> Contact {
    contact_uk()
}

fn lipsum() -> Vec<u8> {
    const LIPSUM: &str = "Lorem ipsum dolor sit amet consectetur adipiscing elit. Quisque faucibus ex sapien vitae pellentesque sem placerat. In id cursus mi pretium tellus duis convallis. Tempus leo eu aenean sed diam urna tempor. Pulvinar vivamus fringilla lacus nec metus bibendum egestas. Iaculis massa nisl malesuada lacinia integer nunc posuere. Ut hendrerit semper vel class aptent taciti sociosqu. Ad litora torquent per conubia nostra inceptos himenaeos.";
    Vec::from(LIPSUM.as_bytes())
}

pub fn messages() -> Vec<ValidatedMessage> {
    let protos = [
        (contact_uk(), vec![contact_russia(), contact_china()]),
        (contact_uk(), vec![contact_russia(), contact_china()]),
        (contact_uk(), vec![contact_russia(), contact_china()]),
        (
            contact_russia(),
            vec![contact_uk(), contact_china(), contact_usa()],
        ),
        (contact_uk(), vec![contact_russia(), contact_china()]),
    ];

    let now = Utc::now();

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
            time: now + Duration::from_hours(i as u64),
        })
        .collect()
}
