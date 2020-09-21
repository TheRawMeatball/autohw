#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_json;

extern crate chrono;
extern crate num_traits;

use diesel::prelude::*;
use rocket_contrib::templates::Template;

mod actions;
mod auth_guard;
mod endpoints;
mod models;
mod pub_imports;
mod schema;
mod wrapper_types;

#[database("db")]
struct DbConn(PgConnection);

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", endpoints::routes())
}
