use {Error, Result};
use types::*;
use super::Pool;

use chrono::{Datelike, NaiveDate, Utc};

use std::collections::HashMap;

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
    let mut stmt = conn.prepare(query!("SELECT id FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| row.get(0))?)
}

pub fn user(pool: &Pool, username: &str) -> Result<Login> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT username, password, api_key FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| {
        Ok(Login {
            username: row.get(0)?,
            password: row.get(1)?,
            api_key: row.get(2)?,
        })
    })?)
}

pub fn user_by_api_key(pool: &Pool, api_key: &str) -> Result<Option<String>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT username FROM users WHERE api_key = ?1"))?;
    match stmt.query_row(params![api_key], |row| row.get(0)) {
        Ok(key) => Ok(key),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn search_uses(pool: &Pool, username: &str) -> Result<(u32, u32)> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT default_uses, bang_uses FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| Ok((row.get(0)?, row.get(1)?)))?)
}

pub fn bangs(pool: &Pool, username: &str) -> Result<Vec<Bang>> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id, bang, value, uses FROM bangs WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Bang {
            id: row.get(0)?,
            owner: user_id,
            bang: row.get(1)?,
            value: row.get(2)?,
            uses: row.get(3)?,
        })
    })?;
    let mut bangs = Vec::new();
    for r in rows {
        bangs.push(r?);
    }
    Ok(bangs)
}

pub fn bang(pool: &Pool, username: &str, bang: &str) -> Result<(i32, String)> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id, value FROM bangs WHERE owner = ?1 AND bang = ?2"))?;
    match stmt.query_row(params![user_id, bang], |row| Ok((row.get(0)?, row.get(1)?))) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(stmt.query_row(params![user_id, "default"], |row| Ok((row.get(0)?, row.get(1)?)))?),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn quick_links(pool: &Pool, username: &str) -> Result<Vec<Link>> {
    let user_id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id, name, url FROM quick_links WHERE owner = ?1"))?;
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

pub fn accounts(pool: &Pool, username: &str) -> Result<Vec<Account>> {
    let id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id, name, amount FROM accounts WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![id], |row| {
        let amount: i64 = row.get(2)?;
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            dollars: amount / 100,
            cents: (amount.abs() % 100) as u8,
        })
    })?;
    let mut accounts = Vec::new();
    for r in rows {
        let r = r?;
        if r.name != "__none" {
            accounts.push(r);
        }
    }
    Ok(accounts)
}

pub fn account_id(pool: &Pool, owner: i32, account: &str) -> Result<i64> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id FROM accounts WHERE owner = ?1 AND name = ?2"))?;
    Ok(stmt.query_row(params![owner, account], |row| row.get(0))?)
}

pub fn account(pool: &Pool, owner: i32, account_id: i64) -> Result<Transactions> {
    let conn = pool.get()?;
    let (name, amount): (String, i64) = conn.query_row(query!("SELECT name, amount FROM accounts WHERE id = ?1 AND owner = ?2"),
        params![account_id, owner], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut accounts = HashMap::new();
    accounts.insert(account_id, name.clone());
    let mut stmt = conn.prepare(query!("SELECT f, t, amount, reason, time FROM transactions WHERE owner = ?1 AND (f = ?2 OR t = ?2) ORDER BY time DESC LIMIT 1000"))?;
    let rows = stmt.query_map(params![owner, account_id], |row| {
        let f = row.get(0)?;
        let t = row.get(1)?;
        let from = if let Some(f) = accounts.get(&f) {
            f.clone()
        } else {
            let mut stmt = conn.prepare(query!("SELECT name FROM accounts WHERE owner = ?1 AND id = ?2"))?;
            let s: String = stmt.query_row(params![owner, f], |row| row.get(0))?;
            accounts.insert(f, s.clone());
            s
        };
        let to = if let Some(t) = accounts.get(&t) {
            t.clone()
        } else {
            let mut stmt = conn.prepare(query!("SELECT name FROM accounts WHERE owner = ?1 AND id = ?2"))?;
            let s: String = stmt.query_row(params![owner, t], |row| row.get(0))?;
            accounts.insert(t, s.clone());
            s
        };
        let amount: i64 = row.get(2)?;
        let date: chrono::DateTime<chrono::Utc> = row.get(4)?;
        Ok(Transaction {
            from: if from == "__none" { "PAYMENT".to_string() } else { from },
            to: if to == "__none" { "EXPENSE".to_string() } else { to },
            dollars: amount / 100,
            cents: (amount % 100) as u8,
            reason: row.get(3)?,
            date: date.format("%a %b %e %Y @ %T").to_string(),
        })
    })?;
    let mut transactions = Vec::new();
    for r in rows {
        transactions.push(r?);
    }

    Ok(Transactions {
        account: name,
        id: account_id,
        dollars: amount / 100,
        cents: (amount % 100) as u8,
        transactions,
    })
}

pub fn reminders(pool: &Pool, username: &str) -> Result<Reminders> {
    let now = Utc::now();

    let owner = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT reason, date FROM reminders WHERE owner = ?1 AND recurrence = ?2 AND date >= date() AND date <= date('now', '+1 month') ORDER BY date"))?;
    let rows = stmt.query_map(params![owner, Recurrence::None as i32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut non_recurring: Vec<(String, NaiveDate)>  = Vec::new();
    for r in rows {
        non_recurring.push(r?);
    }

    let mut stmt = conn.prepare(query!("SELECT reason, date FROM reminders WHERE owner = ?1 AND recurrence = ?2"))?;
    let rows = stmt.query_map(params![owner, Recurrence::Day as i32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut day: Vec<(String, NaiveDate)>  = Vec::new();
    for r in rows {
        day.push(r?);
    }

    let mut stmt = conn.prepare(query!("SELECT reason, date FROM reminders WHERE owner = ?1 AND recurrence = ?2"))?;
    let rows = stmt.query_map(params![owner, Recurrence::Week as i32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut week: Vec<(String, NaiveDate)>  = Vec::new();
    for r in rows {
        week.push(r?);
    }
    week.sort_unstable_by(|(_, datel), (_, dater)| {
        datel.weekday().number_from_monday().partial_cmp(&dater.weekday().number_from_monday()).unwrap()
    });
    let p = week.partition_point(|(_, date)| date.weekday().number_from_monday() < now.weekday().number_from_monday());
    week.rotate_left(p);
    let week = week.into_iter().map(|(r, d)| (r, d.weekday().to_string())).collect();

    let mut stmt = conn.prepare(query!("SELECT reason, date FROM reminders WHERE owner = ?1 AND recurrence = ?2"))?;
    let rows = stmt.query_map(params![owner, Recurrence::Month as i32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut m: Vec<(String, NaiveDate)>  = Vec::new();
    for r in rows {
        m.push(r?);
    }
    m.sort_unstable_by(|(_, datel), (_, dater)| {
        datel.day().partial_cmp(&dater.day()).unwrap()
    });
    let p = m.partition_point(|(_, date)| date.day() < now.day());
    let mut month = Vec::with_capacity(m.len());
    for (i, (r, d)) in m.into_iter().enumerate() {
        if i < p {
            month.push((r, format!("{} {}", (now + ::chrono::Months::new(1)).format("%B"), d.format("%-d"))));
        } else {
            month.push((r, format!("{} {}", now.format("%B"), d.format("%-d"))));
        }
    }
    month.rotate_left(p);

    let mut stmt = conn.prepare(query!("SELECT reason, date FROM reminders WHERE owner = ?1 AND recurrence = ?2"))?;
    let rows = stmt.query_map(params![owner, Recurrence::Year as i32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut year: Vec<(String, NaiveDate)>  = Vec::new();
    for r in rows {
        let (reason, date): (String, NaiveDate) = r?;
        if date.month() == now.month() {
            year.push((reason, date));
        } else if date.month() == now.month() + 1 && (date.day() as i32 + 30 - now.day() as i32).abs() <= 30 {
            year.push((reason, date));
        }
    }
    year.sort_unstable_by(|(_, datel), (_, dater)| {
        datel.ordinal().partial_cmp(&dater.ordinal()).unwrap()
    });
    let year = year.into_iter().map(|(r, d)| (r, d.format("%B %-d").to_string())).collect();

    Ok(Reminders {
        non_recurring,
        day,
        week,
        month,
        year,
    })
}

pub fn inventory(pool: &Pool, username: &str) -> Result<Vec<Item>> {
    let id = user_id(pool, username)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id, name, quantity, unit, low_reminder FROM inventory WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![id], |row| {
        Ok(Item {
            id: row.get(0)?,
            owner: 0,
            name: row.get(1)?,
            quantity: row.get(2)?,
            unit: row.get(3)?,
            low_reminder: row.get(4)?,
        })
    })?;
    let mut inventory = Vec::new();
    for r in rows {
        inventory.push(r?);
    }
    Ok(inventory)
}
