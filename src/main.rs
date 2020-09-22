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

mod actions;
mod auth_guard;
mod endpoints;
mod handlebars_helpers;
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
        .attach(handlebars_helpers::helpers())
        .mount("/", endpoints::routes())
}
