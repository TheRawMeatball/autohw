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
use rocket_contrib::templates::{handlebars, Template};

mod actions;
mod auth_guard;
mod endpoints;
mod models;
mod pub_imports;
mod schema;
mod wrapper_types;

#[database("db")]
struct DbConn(PgConnection);

use self::handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
};

fn percent_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    if let (Some(val), Some(all)) = (h.param(0), h.param(1)) {
        let val = val.value().render().as_str().parse::<f64>().unwrap();
        let all = all.value().render().as_str().parse::<f64>().unwrap();
        out.write(&(100.0 * val / all).to_string())?;
    }

    Ok(())
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("percent", Box::new(percent_helper));
        }))
        .mount("/", endpoints::routes())
}
