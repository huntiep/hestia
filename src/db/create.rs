use Result;
use types::*;
use super::Pool;

use chrono::{DateTime, Utc};

pub fn user(pool: &Pool, user: &Login, default_bang: String) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO users (username, password, api_key, default_uses, bang_uses) VALUES (?1, ?2, ?3, 0, 0)"),
        params![user.username, user.password, user.api_key])?;
    let owner = super::read::user_id(pool, &user.username)?;
    let def_bang = NewBang {
        owner: owner,
        bang: String::from("default"),
        value: default_bang,
    };
    bang(pool, &def_bang)?;
    account(pool, NewAccount { name: "__none".to_string(), owner: owner })
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

pub fn account(pool: &Pool, account: NewAccount) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO accounts (owner, name) VALUES (?1, ?2)"),
        params![account.owner, account.name])?;
    Ok(())
}

pub fn transaction(pool: &Pool, transaction: NewTransaction) -> Result<()> {
    let from = super::read::account_id(pool, transaction.owner, &transaction.from)?;
    let to = super::read::account_id(pool, transaction.owner, &transaction.to)?;
    let time: DateTime<Utc> = Utc::now();
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let txid: u32 = tx.query_row(query!("INSERT INTO transactions (owner, f, t, amount, reason, time) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id"),
        params![transaction.owner, from, to, transaction.dollars*100 + transaction.cents as i64, transaction.reason, time],
        |row| row.get(0))?;
    {
        tx.execute(query!("UPDATE accounts SET amount = amount - (SELECT amount FROM transactions WHERE id = ?1) WHERE id = ?2"),
            params![txid, from])?;
        tx.execute(query!("UPDATE accounts SET amount = amount + (SELECT amount FROM transactions WHERE id = ?1) WHERE id = ?2"),
            params![txid, to])?;
    }
    Ok(tx.commit()?)
}

pub fn reminder(pool: &Pool, owner: i32, reminder: Reminder) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO reminders (owner, recurrence, reason, date) VALUES (?1, ?2, ?3, ?4)"),
        params![owner, reminder.recurrence as i32, reminder.reason, reminder.date])?;
    Ok(())
}
