use bcrypt::{self, DEFAULT_COST};
use hayaku::Request;

pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl NewUser {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (username, email) = form_values!(req, "username", "email");
        let (password, confirm) = form_values!(req, "password", "password_confirm");

        if password != confirm {
            return None;
        }

        let password_hash = try_opt!(bcrypt::hash(&password, DEFAULT_COST).ok());
        Some(NewUser {
            username: username,
            email: email,
            password: password_hash,
        })
    }
}

pub struct Login {
    pub username: String,
    pub password: String,
}

impl Login {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (username, password) = form_values!(req, "username", "password");

        Some(Login {
            username: username,
            password: password,
        })
    }
}

pub struct UpdatePassword {
    pub password: String,
}

impl UpdatePassword {
    pub fn new(req: &mut Request, old_password_hash: String) -> Option<Self> {
        let (old, new, new_confirm) = form_values!(req, "ld_password", "new_password", "confirm_password");

        if new != new_confirm {
            return None;
        }

        if !try_opt!(bcrypt::verify(&old, &old_password_hash).ok()) {
            return None;
        }
        let password_hash = try_opt!(bcrypt::hash(&new, DEFAULT_COST).ok());
        Some(UpdatePassword {
            password: password_hash,
        })
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
}

impl Bang {
    pub fn new(req: &mut Request, owner: i32, id: i64) -> Option<Self> {
        let (bang, value) = form_values!(req, "bang", "value");

        Some(Bang {
            id: id,
            owner: owner,
            bang: bang,
            value: value,
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

