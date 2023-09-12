use db;
use templates::*;
use types::*;

route!{home, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);
    let accounts = db::read::accounts(&ctx.db_pool, username)?;
    let body = FinanceHomeTmpl { accounts };
    tmpl!(req, res, ctx, None, body);
}}

// POST /account
route!{new_account, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let account = NewAccount::new(req, owner);
    if account.is_none() {
        redirect!(res, ctx, "finance", "Invalid input");
    }
    db::create::account(pool, account.unwrap())?;
    redirect!(res, ctx, "finance", "Account created");
}}

// GET /account/{id}
route!{view_account, req, res, ctx, {
    let cookies = &req.get_cookies();
    let username = check_login!(&cookies, res, ctx);
    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let account = parse_param!(req, res, ctx, "id", i64);
    let transactions = db::read::account(pool, owner, account)?;
    let name = transactions.account.clone();
    let body = FinanceAccountTmpl { transactions };
    tmpl!(req, res, ctx, Some(&name), body);
}}

// POST /account/{id}
route!{edit_account, req, res, ctx, {
    todo!();
}}

// GET /account/{id}/delete
route!{delete_account, req, res, ctx, {
    todo!();
}}

// POST /transaction
route!{new_transaction, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let transaction = NewTransaction::new(req, owner);
    if transaction.is_none() {
        redirect!(res, ctx, "finance", "Invalid input");
    }
    db::create::transaction(pool, transaction.unwrap())?;
    redirect!(res, ctx, "finance", "Transaction processed");
}}
