use Result;
use super::Pool;

pub fn bang(pool: &Pool, username: &str, id: i64) -> Result<()> {
    let owner = super::read::user_id(pool, username)?;
    let conn = pool.get()?;
    conn.execute(query!("DELETE FROM bangs WHERE owner = ?1 AND id = ?2"),
        params![owner, id])?;
    Ok(())
}

pub fn quick_link(pool: &Pool, username: &str, id: i64) -> Result<()> {
    let owner = super::read::user_id(pool, username)?;
    let conn = pool.get()?;
    conn.execute(query!("DELETE FROM quick_links WHERE owner = ?1 AND id = ?2"),
        params![owner, id])?;
    Ok(())
}
