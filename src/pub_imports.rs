pub(crate) use crate::actions;
pub(crate) use crate::auth_guard::*;
pub(crate) use crate::models;
pub(crate) use crate::schema;
pub(crate) use crate::wrapper_types::*;
pub(crate) use crate::DbConn;

pub use rocket::http::CookieJar;
pub use rocket::request::LenientForm;

use chrono::{self, Duration, NaiveDate, Utc};
#[inline]
pub fn now() -> NaiveDate {
    let now = Utc::now().naive_local() - Duration::seconds(5 * 3600 + 1800);
    println!("{}", now);
    now.date()
}
