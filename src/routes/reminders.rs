use db;
use types::*;

// POST /new
route!{new_reminder, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let reminder = Reminder::new(req);
    if reminder.is_none() {
        redirect!(res, ctx, "", "Invalid input");
    }
    db::create::reminder(pool, owner, reminder.unwrap())?;
    redirect!(res, ctx, "", "Reminder created");
}}
