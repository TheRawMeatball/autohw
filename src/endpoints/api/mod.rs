use crate::pub_imports::*;

mod class;
mod homework;
mod user;

pub fn routes() -> Vec<rocket::Route> {
    vec![]
        .add(user::routes())
        .add(homework::routes())
        .add(class::routes())
        .set_root("/api")
}
