use db;
use types::*;
use templates::*;

// GET /inventory
route!{home, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);
    let inventory = db::read::inventory(&ctx.db_pool, username)?;
    let body = InventoryHomeTmpl { inventory };
    tmpl!(req, res, ctx, None, body);
}}

// POST /inventory/new
route!{new_item, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let item = Item::new(req, owner);
    if item.is_none() {
        redirect!(res, ctx, "inventory", "Invalid input");
    }
    db::create::inventory_item(pool, item.unwrap())?;
    redirect!(res, ctx, "inventory", "Item created");
}}

// POST /inventory/item/{id}
route!{edit_item, req, res, ctx, {
    let cookies = &req.get_cookies();
    let username = check_login!(&cookies, res, ctx);
    let pool = &ctx.db_pool;
    let owner = db::read::user_id(pool, username)?;
    let item = parse_param!(req, res, ctx, "id", i32);
    let quantity = req.form_value("quantity");
    if quantity.is_none() {
        redirect!(res, ctx, "inventory", "Invalid input");
    }
    let quantity = quantity.unwrap().parse::<i32>();
    if quantity.is_err() {
        redirect!(res, ctx, "inventory", "Invalid input");
    }
    db::update::inventory_set_quantity(pool, owner, item, quantity.unwrap())?;
    redirect!(res, ctx, "inventory", "Item quantity changed");
}}
