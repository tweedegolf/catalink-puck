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
        contact_india(),
        contact_china(),
        contact_russia(),
        contact_usa(),
        contact_uk(),
        contact_france(),
    ]
}

pub fn our_contact() -> Contact {
    contact_pakistan()
}

pub fn messages() -> Vec<ValidatedMessage> {
    vec![]
}
