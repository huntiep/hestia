use db;
use templates::*;
use types::*;

// GET /settings
route!{settings, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let bangs = db::read::bangs(pool, username)?;
    let links = db::read::quick_links(pool, username)?;
    let user = db::read::user(pool, username)?;
    let body = SettingsTmpl {
        bangs: bangs,
        links: links,
        api_key: user.api_key,
    };
    let tmpl = Template::new(Some("Settings"), body);
    Ok(res.fmt_body(tmpl))
}}

// POST /settings/password
route!{password, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let user = db::read::user(pool, username)?;
    let password = Login::change_password(req, username.to_string(), user.password);
    if password.is_none() {
        redirect!(res, ctx, "settings", "Invalid data");
    }
    let password = password.unwrap();

    db::update::password(&ctx.db_pool, &password)?;

    redirect!(res, ctx, "settings", "Password updated");
}}

// GET /settings/new-api-key
route!{new_api_key, req, res, ctx, {
    db::update::new_api_key(&ctx.db_pool, &check_login!(&req.get_cookies(), res, ctx))?;
    redirect!(res, ctx, "settings", "Api key changed");
}}

// POST /settings/bangs
route!{create_bang, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let bang = NewBang::new(req, owner);
    if bang.is_none() {
        redirect!(res, ctx, "settings", "Invalid input");
    }
    let bang = bang.unwrap();
    db::create::bang(pool, &bang)?;
    redirect!(res, ctx, "settings", "Bang created");
}}

// POST /settings/bangs/{id}
route!{edit_bang, req, res, ctx, {
    let id = parse_param!(req, res, ctx, "id", i64);

    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let bang = Bang::new(req, owner, id);
    if bang.is_none() {
        redirect!(res, ctx, "settings", "Invalid data");
    }
    let bang = bang.unwrap();
    db::update::bang(pool, &bang)?;
    redirect!(res, ctx, "settings", "Bang updated");
}}

// GET /settings/bangs/{id}/delete
route!{delete_bang, req, res, ctx, {
    let id = parse_param!(req, res, ctx, "id", i64);

    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    db::delete::bang(&ctx.db_pool, username, id)?;
    redirect!(res, ctx, "settings", "Bang deleted if it existed");
}}

// POST /settings/links
route!{create_link, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let link = NewLink::new(req, owner);
    if link.is_none() {
        redirect!(res, ctx, "settings", "Invalid data");
    }
    let link = link.unwrap();
    db::create::quick_link(pool, &link)?;
    redirect!(res, ctx, "settings", "Link created");
}}

// POST /settings/links/{id}
route!{edit_link, req, res, ctx, {
    let id = parse_param!(req, res, ctx, "id", i64);

    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let link = Link::new(req, owner, id);
    if link.is_none() {
        redirect!(res, ctx, "settings", "Invalid data");
    }
    let link = link.unwrap();
    db::update::quick_link(pool, &link)?;
    redirect!(res, ctx, "settings", "Link updated");
}}

// GET /settings/links/{id}/delete
route!{delete_link, req, res, ctx, {
    let id = parse_param!(req, res, ctx, "id", i64);

    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    db::delete::quick_link(&ctx.db_pool, username, id)?;
    redirect!(res, ctx, "settings", "Link deleted if it existed");
}}
