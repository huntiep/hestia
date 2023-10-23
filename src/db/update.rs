use Result;
use types::*;
use super::Pool;

use chrono::Utc;

pub fn password(pool: &Pool, password: &Login) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("UPDATE users SET password = ?1 WHERE username = ?2"),
        params![password.password, password.username])?;
    Ok(())
}

pub fn new_api_key(pool: &Pool, username: &str) -> Result<()> {
    let conn = pool.get()?;
    let api_key = Login::gen_api_key();
    conn.execute(query!("UPDATE users SET api_key = ?1 WHERE username = ?2"),
        params![api_key, username])?;
    Ok(())
}

pub fn search_uses(pool: &Pool, username: &str, bang: i32, defaultp: bool) -> Result<()> {
    let conn = pool.get()?;
    if defaultp {
        conn.execute(query!("UPDATE users SET default_uses = default_uses + 1 WHERE username = ?1"),
            params![username])?;
    } else {
        conn.execute(query!("UPDATE users SET bang_uses = bang_uses + 1 WHERE username = ?1"),
            params![username])?;
    }
    conn.execute(query!("UPDATE bangs SET uses = uses + 1 WHERE id = ?1"),
        params![bang])?;
    Ok(())
}

pub fn bang(pool: &Pool, bang: &Bang) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("UPDATE bangs SET bang = ?1, value = ?2 WHERE owner = ?3 AND id = ?4"),
        params![bang.bang, bang.value, bang.owner, bang.id])?;
    Ok(())
}

pub fn quick_link(pool: &Pool, link: &Link) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("UPDATE quick_links SET name = ?1, url = ?2 WHERE owner = ?3 AND id = ?4"),
        params![link.name, link.url, link.owner, link.id])?;
    Ok(())
}

pub fn inventory_set_quantity(pool: &Pool, owner: i32, item_id: i32, quantity: i32) -> Result<()> {
    let conn = pool.get()?;
    let (name, low_reminder): (String, i32) = conn.query_row(query!("UPDATE inventory SET quantity = ?3 WHERE owner = ?1 and id = ?2 RETURNING name, low_reminder"),
        params![owner, item_id, quantity],
        |row| Ok((row.get(0)?, row.get(1)?)))?;
    if quantity <= low_reminder {
        let reminder = Reminder { recurrence: Recurrence::None, reason: format!("Buy more {}", name), date: Utc::now().date_naive() + ::chrono::Days::new(7) };
        super::create::reminder(pool, owner, reminder)?;
    }
    Ok(())
}
