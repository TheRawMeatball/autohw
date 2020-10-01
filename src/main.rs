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
use rocket::fairing::AdHoc;
use rocket_contrib::serve::StaticFiles;

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
        .attach(AdHoc::on_attach(
            "Static Files config",
            |mut rocket| async {
                let files_path = rocket
                    .config()
                    .await
                    .get_string("static_files_path")
                    .expect("missing static files path!");
                Ok(rocket.mount("/static", StaticFiles::from(files_path)))
            },
        ))
        .mount("/", endpoints::routes())
}
