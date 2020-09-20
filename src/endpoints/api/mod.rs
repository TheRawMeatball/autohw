use crate::pub_imports::*;

mod homework;
mod user;

pub fn routes() -> Vec<rocket::Route> {
    user::routes().add(homework::routes()).set_root("/api")
}
