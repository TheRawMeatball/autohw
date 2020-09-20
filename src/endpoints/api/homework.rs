use crate::pub_imports::*;
use actions::homework::{self, HomeworkApiError};
use models::homework::*;
use rocket::response::{Flash, Redirect};

pub fn routes() -> Vec<rocket::Route> {
    routes![add].set_root("/homework")
}

#[post("/add", data = "<model>")]
async fn add(
    user: AuthUser,
    model: Form<AddHomeworkModel>,
    conn: DbConn,
) -> Result<Redirect, Flash<Redirect>> {
    match conn
        .run(move |c| homework::add_homework(&*model, user, c))
        .await
    {
        Ok(_) => Ok(Redirect::to("/")),
        Err(HomeworkApiError::UnspecifiedTime) => Err(Flash::error(
            Redirect::to("/"),
            "Specify due date or set as repeating",
        )),
        Err(_) => Err(Flash::error(Redirect::to("/"), "Server error")),
    }
}
