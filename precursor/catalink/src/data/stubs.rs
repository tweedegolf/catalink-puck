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
        contact_usa(),
        contact_russia(),
        contact_france(),
        contact_uk(),
        contact_india(),
        contact_pakistan(),
    ]
}

pub fn our_contact() -> Contact {
    contact_china()
}

pub fn messages() -> Vec<ValidatedMessage> {
    let group = vec![contact_usa(), contact_uk(), contact_france(), contact_russia(), contact_china()];
    let protos = [
        (contact_usa(), group.clone(), "Seismic event detected 70 km west of Ürümqi, signature consistent with possible test. Requesting China to clarify."),
        (contact_china(), group.clone(), "No test conducted by our forces. Checking natural seismic activity in that region."),
        (contact_russia(), group.clone(), "Concur, recommend geological verification before further inquiry. No test."),
        (contact_uk(), group.clone(), "Will cross-check against IMS station data, will share results."),
        (contact_france(), group.clone(), "Awaiting UK data. No change in our posture."),
    ];

    let now = Utc::now();

    protos
        .into_iter()
        .enumerate()
        .map(|(i, (sender, recipients, body))| ValidatedMessage {
            sender: sender.name.clone(),
            recipients: recipients
                .into_iter()
                .filter(|recipient| *recipient != sender)
                .map(|recipient| recipient.name)
                .collect(),
            body: body.as_bytes().to_vec(),
            time: now + Duration::from_hours(i as u64),
        })
        .collect()
}
