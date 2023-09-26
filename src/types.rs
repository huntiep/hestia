use bcrypt::{self, DEFAULT_COST};
use chrono::NaiveDate;
use hayaku::Request;
use rand::Rng;
use rand::distributions::Alphanumeric;

pub struct Login {
    pub username: String,
    pub password: String,
    pub api_key: String,
}

impl Login {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (username, password) = form_values!(req, "username", "password");

        Some(Login {
            username: username,
            password: password,
            api_key: String::new(),
        })
    }

    pub fn new_user(req: &mut Request) -> Option<Self> {
        let (username, password, confirm) = form_values!(req, "username", "password", "password_confirm");

        if password != confirm {
            return None;
        }

        let password_hash = try_opt!(bcrypt::hash(&password, DEFAULT_COST).ok());
        Some(Login {
            username: username,
            password: password_hash,
            api_key: Self::gen_api_key(),
        })
    }

    pub fn change_password(req: &mut Request, username: String, old_password_hash: String) -> Option<Self> {
        let (old, new, new_confirm) = form_values!(req, "ld_password", "new_password", "confirm_password");

        if new != new_confirm {
            return None;
        }

        if !try_opt!(bcrypt::verify(&old, &old_password_hash).ok()) {
            return None;
        }
        let password_hash = try_opt!(bcrypt::hash(&new, DEFAULT_COST).ok());
        Some(Login {
            username: username,
            password: password_hash,
            api_key: String::new(),
        })
    }

    pub fn gen_api_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(50)
            .map(char::from)
            .collect()
    }
}

pub struct NewBang {
    pub owner: i32,
    pub bang: String,
    pub value: String,
}

impl NewBang {
    pub fn new(req: &mut Request, owner: i32) -> Option<Self> {
        let (bang, value) = form_values!(req, "bang", "value");

        Some(NewBang {
            owner: owner,
            bang: bang,
            value: value,
        })
    }
}

pub struct Bang {
    pub id: i64,
    pub owner: i32,
    pub bang: String,
    pub value: String,
    pub uses: i32,
}

impl Bang {
    pub fn new(req: &mut Request, owner: i32, id: i64) -> Option<Self> {
        let (bang, value) = form_values!(req, "bang", "value");

        Some(Bang {
            id: id,
            owner: owner,
            bang: bang,
            value: value,
            uses: 0,
        })
    }
}

pub struct NewLink {
    pub owner: i32,
    pub name: String,
    pub url: String,
}

impl NewLink {
    pub fn new(req: &mut Request, owner: i32) -> Option<Self> {
        let (name, url) = form_values!(req, "name", "url");

        Some(NewLink {
            owner: owner,
            name: name,
            url: url,
        })
    }
}

pub struct Link {
    pub id: i64,
    pub owner: i32,
    pub name: String,
    pub url: String,
}

impl Link {
    pub fn new(req: &mut Request, owner: i32, id: i64) -> Option<Self> {
        let (name, url) = form_values!(req, "name", "url");

        Some(Link {
            id: id,
            owner: owner,
            name: name,
            url: url,
        })
    }
}

pub struct NewAccount {
    pub name: String,
    pub owner: i32,
}

impl NewAccount {
    pub fn new(req: &mut Request, owner: i32,) -> Option<Self> {
        let name = form_values!(req, "name");

        Some(NewAccount {
            name,
            owner,
        })
    }
}

pub struct Account {
    pub id: i64,
    pub name: String,
    pub dollars: i64,
    pub cents: u8,
}

pub struct NewTransaction {
    pub owner: i32,
    pub from: String,
    pub to: String,
    pub dollars: i64,
    pub cents: u8,
    pub reason: String,
}

impl NewTransaction {
    pub fn new(req: &mut Request, owner: i32,) -> Option<Self> {
        let (from, to, amount) = form_values!(req, "from", "to", "amount");
        let reason = req.form_value("reason")?;
        let amount: Vec<_> = amount.split('.').collect();
        let (dollars, cents) = if amount.len() > 2 {
            return None;
        } else if amount.len() == 1 {
            (amount[0].parse().ok()?, 0)
        } else {
            (amount[0].parse().ok()?, amount[1].parse().ok()?)
        };

        Some(NewTransaction {
            owner,
            from,
            to,
            dollars,
            cents,
            reason,
        })
    }
}

pub struct Transaction {
    pub from: String,
    pub to: String,
    pub dollars: i64,
    pub cents: u8,
    pub reason: String,
    pub date: String,
}

pub struct Transactions {
    pub account: String,
    pub id: i64,
    pub dollars: i64,
    pub cents: u8,
    pub transactions: Vec<Transaction>,
}

pub enum Recurrence {
    None = 0,
    Day = 1,
    Week = 2,
    Month = 3,
    Year = 4,
}

pub struct Reminder {
    pub recurrence: Recurrence,
    pub reason: String,
    pub date: NaiveDate,
}

impl Reminder {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (reason, date, recurrence) = form_values!(req, "reason", "date", "recurrence");
        let recurrence = match recurrence.as_str() {
            "none" => Recurrence::None,
            "day" => Recurrence::Day,
            "week" => Recurrence::Week,
            "month" => Recurrence::Month,
            "year" => Recurrence::Year,
            _ => return None,
        };
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;

        Some(Reminder {
            recurrence,
            reason,
            date: date.into(),
        })
    }
}

pub struct Reminders {
    pub non_recurring: Vec<(String, NaiveDate)>,
    pub day: Vec<(String, NaiveDate)>,
    pub week: Vec<(String, String)>,
    pub month: Vec<(String, String)>,
    pub year: Vec<(String, String)>,
}
