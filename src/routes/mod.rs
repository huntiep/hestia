use {db, Context, Error};
use templates::*;
use types::*;

pub mod finance;
pub mod reminders;
pub mod settings;
mod util;

use hayaku::{Request, Response, Status};
use hayaku::header::{self, HeaderValue};

route!{home, req, res, ctx, {
    let cookies = &req.get_cookies();
    if let Some(username) = util::check_login(ctx, cookies)? {
        let links = db::read::quick_links(&ctx.db_pool, username)?;
        let search_uses = db::read::search_uses(&ctx.db_pool, username)?;
        let reminders = db::read::reminders(&ctx.db_pool, username)?;
        let api_key = db::read::user(&ctx.db_pool, username)?.api_key;
        let body = HomeTmpl { links, search_uses, reminders, api_key };
        tmpl!(req, res, ctx, Some("Home"), body);
    } else {
        let body = include_str!("../../templates/login.html");
        tmpl!(req, res, ctx, Some("Login"), body);
    }
}}

// GET /signup
route!{signup, req, res, ctx, {
    if !ctx.signup {
        return not_found(req, res, ctx);
    } else if util::check_login(ctx, &req.get_cookies())?.is_some() {
        redirect!(res, ctx, "", "You already have an account");
    } else {
        let body = include_str!("../../templates/signup.html");
        tmpl!(req, res, ctx, Some("Signup"), body);
    }
}}

// POST /signup
route!{signup_post, req, res, ctx, {
    if !ctx.signup {
        return not_found(req, res, ctx);
    } else if util::check_login(ctx, &req.get_cookies())?.is_some() {
        redirect!(res, ctx, "", "You already have an account");
    }

    let new_user = if let Some(user) = Login::new_user(req) {
        user
    } else {
        redirect!(res, ctx, "signup", "Signup failed");
    };

    db::create::user(&ctx.db_pool, &new_user, ctx.default_bang.clone())?;
    util::login(new_user.username, &mut res.cookies(), ctx)?;
    redirect!(res, ctx, "", "Signup successful");
}}

// POST /login
route!{login, req, res, ctx, {
    if util::check_login(ctx, &req.get_cookies())?.is_some() {
        redirect!(res, ctx, "", "You are already logged in");
    }

    let login = if let Some(login) = Login::new(req) {
        login
    } else {
        redirect!(res, ctx, "", "Login failed");
    };

    if !db::read::check_login(&ctx.db_pool, &login)? {
        redirect!(res, ctx, "", "Login failed");
    }

    util::login(login.username, &mut res.cookies(), ctx)?;
    redirect!(res, ctx, "", "Login successful");
}}

// GET /logout
route!{logout, req, res, ctx, {
    util::logout(&req.get_cookies(), &mut res.cookies(), ctx);
    redirect!(res, ctx, "", "Logout successful");
}}

// POST /search/{api-key}
route!{search, req, res, ctx, {
    let api_key = req.get_param("api-key");
    let username = if let Some(u) = db::read::user_by_api_key(&ctx.db_pool, &api_key)? {
        u
    } else {
        redirect!(res, ctx, "", "Invalid content");
    };

    let search = if let Some(s) = req.form_value("q") {
        s
    } else {
        redirect!(res, ctx, "", "Invalid content");
    };

    if search.starts_with('!') {
        let terms: Vec<&str> = search.splitn(2, ' ').collect();
        let (bang, search): (&str, &str) = (&terms[0][1..], terms[1]);
        let (bang_id, bang) = db::read::bang(&ctx.db_pool, &username, bang)?;
        db::update::search_uses(&ctx.db_pool, &username, bang_id, false)?;
        let url = bang + search;
        ok!(res.redirect(Status::FOUND, &url, "You are being redirected"));
    } else {
        let (bang_id, bang) = db::read::bang(&ctx.db_pool, &username, "default")?;
        db::update::search_uses(&ctx.db_pool, &username, bang_id, true)?;
        ok!(res.redirect(Status::TEMPORARY_REDIRECT, &bang, "You are being redirected"));
    }
}}

// GET /opensearch/{api-key}/opensearch.xml
route!{opensearch, req, res, ctx, {
    let api_key = req.get_param("api-key");
    let tmpl = include_str!("../../opensearch.xml");
    let mut tmpl: Vec<_> = tmpl.split("APIKEY").collect();
    tmpl.insert(1, &api_key);
    Ok(res.body(tmpl.into_iter().collect::<String>()))
}}

route!{not_found, req, res, ctx, {
    res.status(Status::NOT_FOUND);
    res.add_header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    ok!(res.body(include_str!("../../templates/404.html")));
}}

pub fn internal_error(_req: &mut Request, res: &mut Response, _ctx: &Context, err: &Error) {
    res.status(Status::INTERNAL_SERVER_ERROR);
    res.add_header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    match *err {
        Error::Sqlite(_) | Error::R2D2(_) => res.body(format!("{}{:?}{}", include_str!("../../templates/sqlite_error.html"),
                                                                        err,
                                                                        include_str!("../../templates/foot.html"))),
        _=> res.body(include_str!("../../templates/internal_error.html")),
    }
}
