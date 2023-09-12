#[macro_use]
extern crate bart_derive;
extern crate bcrypt;
extern crate brotli;
#[macro_use]
extern crate check_psql;
extern crate chrono;
#[macro_use]
extern crate hayaku;
extern crate libflate;
#[macro_use]
extern crate quick_error;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rand;
#[macro_use]
extern crate rusqlite;
extern crate rusqlite_migration;
#[macro_use]
extern crate serde_derive;
extern crate sessions;
extern crate time;
extern crate toml;

#[macro_use]
mod macros;
mod db;
mod routes;
mod templates;
mod types;

use routes::*;

use hayaku::{Http, Router};
use r2d2_sqlite::SqliteConnectionManager;

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = ::std::result::Result<T, Error>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Bcrypt(err: ::bcrypt::BcryptError) {
            from()
        }
        Io(err: ::std::io::Error) {
            from()
        }
        ParseInt(err: ::std::num::ParseIntError) {
            from()
        }
        R2D2(err: r2d2::Error) {
            from()
        }
        Session(err: sessions::Error) {
            from()
        }
        Sqlite(err: rusqlite::Error) {
            from()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    db_path: PathBuf,
    sessions_dir: PathBuf,
    addr: Option<SocketAddr>,
    mount: Option<String>,
    signup: bool,
    default_bang: String,
}

pub struct Context {
    pub db_pool: db::Pool,
    pub logins: Arc<Mutex<sessions::SessionSet>>,
    pub name: String,
    pub mount: String,
    pub signup: bool,
    pub default_bang: String,
}

fn main() {
    let config_path = "hestia.toml";
    let buf = fs::read_to_string(config_path).expect("failed to read config");
    let config: Config = toml::from_str(&buf).expect("failed to parse config");

    let manager = SqliteConnectionManager::file(config.db_path);
    let pool = r2d2::Pool::new(manager).expect("Failed to create db pool");

    {
        // Run migrations
        use rusqlite_migration::{M, Migrations};
        let migrations = Migrations::new(vec![
            M::up(include_str!("../migrations/1/up.sql"))
                .down(include_str!("../migrations/1/down.sql")),
            M::up(include_str!("../migrations/2/up.sql"))
                .down(include_str!("../migrations/2/down.sql")),
        ]);
        let mut conn = pool.get().unwrap();
        migrations.to_latest(&mut conn).unwrap();
    }

    let mount = match config.mount {
        Some(m) => if m.ends_with('/') {
            m
        } else {
            m + "/"
        },
        None => "/".to_string(),
    };

    let sessions = if config.sessions_dir.exists() && config.sessions_dir.is_dir() {
        ::sessions::SessionSet::load(config.sessions_dir).expect("failed to load sessions")
    } else {
        ::sessions::SessionSet::new(config.sessions_dir).expect("failed to load sessions")
    };

    let ctx = Context {
        db_pool: pool,
        logins: Arc::new(Mutex::new(sessions)),
        name: "Jane".to_string(),
        mount: mount,
        signup: config.signup,
        default_bang: config.default_bang,
    };

    let mut router = Router::mount(ctx.mount.clone());
    router.set_not_found_handler(Arc::new(not_found));
    router.set_internal_error_handler(Arc::new(internal_error));
    router! {
        router,
        get "/" => home,
        get "/signup" => signup,
        post "/signup" => signup_post,
        post "/login" => login,
        get "/logout" => logout,

        // settings
        get "/settings" => settings::settings,
        get "/settings/new-api-key" => settings::new_api_key,
        post "/settings/password" => settings::password,
        post "/settings/bangs" => settings::create_bang,
        post "/settings/bangs/{id:[[:digit:]]+}" => settings::edit_bang,
        get "/settings/bangs/{id:[[:digit:]]+}" => settings::delete_bang,
        post "/settings/links" => settings::create_link,
        post "/settings/links/{id:[[:digit:]]+}" => settings::edit_link,
        get "/settings/links/{id:[[:digit:]]+}" => settings::delete_link,

        // search
        post "/search" => search,
        post "/opensearch.xml" => opensearch,

        // finance
        get "/finance" => finance::home,
        post "/finance/account" => finance::new_account,
        get "/finance/account/{id:[[:digit:]]+}" => finance::view_account,
        post "/finance/account/{id:[[:digit:]]+}" => finance::edit_account,
        get "/finance/account/{id:[[:digit:]]+}/delete" => finance::delete_account,
        post "/finance/transaction" => finance::new_transaction,
    }

    let addr = config.addr.unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
    Http::new(router, ctx).listen_and_serve(addr);
}
