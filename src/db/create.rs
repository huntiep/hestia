use Result;
use types::*;
use super::Pool;

pub fn user(pool: &Pool, user: &NewUser, default_bang: String) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO users (username, email, password, default_uses, bang_uses) VALUES (?1, ?2, ?3, 0, 0)"),
        params![user.username, user.email, user.password])?;
    let owner = super::read::user_id(pool, &user.username)?;
    let def_bang = NewBang {
        owner: owner,
        bang: String::from("default"),
        value: default_bang,
    };
    bang(pool, &def_bang)
}

pub fn bang(pool: &Pool, bang: &NewBang) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO bangs (owner, bang, value) VALUES (?1, ?2, ?3)"),
        params![bang.owner, bang.bang, bang.value])?;
    Ok(())
}

pub fn quick_link(pool: &Pool, link: &NewLink) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO quick_links (owner, name, url) VALUES (?1, ?2, ?3)"),
        params![link.owner, link.name, link.url])?;
    Ok(())
}
