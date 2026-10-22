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

fn contact_pakistan() -> Contact {
    Contact {
        name: "Pakistan".to_string(),
    }
}

pub fn contacts() -> Vec<Contact> {
    vec![
        contact_china(),
        contact_uk(),
        contact_france(),
        contact_india(),
        contact_pakistan(),
        contact_russia(),
    ]
}

pub fn our_contact() -> Contact {
    contact_usa()
}

pub fn messages() -> Vec<ValidatedMessage> {
    let protos = [
        (contact_usa(), vec![contact_russia(), contact_china()], "Reporting loss of contact with satellite USA-324. Requesting confirmation of ASAT activity."),
        (contact_russia(), vec![contact_usa(), contact_china()], "No ASAT activity."),
        (contact_china(), vec![contact_russia(), contact_usa()], "No ASAT activity."),
        (contact_usa(), vec![contact_china(), contact_russia()], "Continuing investigation. Stand by for findings."),
    ];

    let now = Utc::now();

    protos
        .into_iter()
        .enumerate()
        .map(|(i, (sender, recipients, body))| ValidatedMessage {
            sender: sender.name,
            recipients: recipients
                .into_iter()
                .map(|recipient| recipient.name)
                .collect(),
            body: body.as_bytes().to_vec(),
            time: now + Duration::from_hours(i as u64),
        })
        .collect()
}
