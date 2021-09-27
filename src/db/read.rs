use {Error, Result};
use types::*;
use super::Pool;

pub fn check_login(pool: &Pool, login: &Login) -> Result<bool> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT password FROM users WHERE username = ?1"))?;
    let password: String = match stmt.query_row(params![login.username], |row| row.get(0)) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
        Err(e) => return Err(Error::from(e)),
    };
    Ok(bcrypt::verify(&login.password, &password)?)
}

pub fn user_id(pool: &Pool, username: &str) -> Result<i32> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT (id) FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| row.get(0))?)
}

pub fn user(pool: &Pool, username: &str) -> Result<NewUser> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT (username, email, password) FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| {
        Ok(NewUser {
            username: row.get(0)?,
            email: row.get(1)?,
            password: row.get(2)?,
        })
    })?)
}

pub fn search_uses(pool: &Pool, username: &str) -> Result<(u32, u32)> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT (default_uses, bang_uses) FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| Ok((row.get(0)?, row.get(1)?)))?)
}

pub fn bangs(pool: &Pool, username: &str) -> Result<Vec<Bang>> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT (id, bang, value) FROM bangs WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Bang {
            id: row.get(0)?,
            owner: user_id,
            bang: row.get(1)?,
            value: row.get(2)?,
        })
    })?;
    let mut bangs = Vec::new();
    for r in rows {
        bangs.push(r?);
    }
    Ok(bangs)
}

pub fn bang(pool: &Pool, username: &str, bang: &str) -> Result<String> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT value FROM bangs WHERE owner = ?1 AND bang = ?2"))?;
    match stmt.query_row(params![user_id, bang], |row| row.get(0)) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(stmt.query_row(params![user_id, "default"], |row| row.get(0))?),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn quick_links(pool: &Pool, username: &str) -> Result<Vec<Link>> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT (id, name, url) FROM quick_links WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Link {
            id: row.get(0)?,
            owner: user_id,
            name: row.get(1)?,
            url: row.get(2)?,
        })
    })?;
    let mut links = Vec::new();
    for r in rows {
        links.push(r?);
    }
    Ok(links)
}
