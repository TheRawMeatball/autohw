use crate::pub_imports::*;
use actions::homework::{self, HomeworkApiError};
use models::homework::*;
use models::hw_progress::*;
use rocket::response::{Flash, Redirect};

pub fn routes() -> Vec<rocket::Route> {
    routes![add, progress, set_weight].set_root("/homework")
}

#[post("/add", data = "<model>")]
async fn add(
    user: AuthUser,
    model: LenientForm<AddHomeworkModel>,
    conn: DbConn,
) -> Result<Redirect, Flash<Redirect>> {
    match conn
        .run(move |c| homework::add_homework(&*model, user, c))
        .await
    {
        Ok(_) => Ok(Redirect::to("/")),
        Err(HomeworkApiError::UnspecifiedTime) => Err(Flash::error(
            Redirect::to("/add"),
            "Specify due date or set as repeating",
        )),
        Err(_) => Err(Flash::error(Redirect::to("/add"), "Server error")),
    }
}

#[post("/progress", data = "<model>")]
async fn progress(
    user: AuthUser,
    model: LenientForm<ChangeProgressModel>,
    conn: DbConn,
) -> Result<Redirect, Flash<Redirect>> {
    match conn
        .run(move |c| homework::change_progress(&user.into(), &*model, c))
        .await
    {
        Ok(_) => Ok(Redirect::to("/")),
        Err(e) => {
            eprintln!("{}", e);
            Err(Flash::error(Redirect::to("/"), "Server error"))
        }
    }
}

#[post("/set-weight", data = "<model>")]
async fn set_weight(user: AuthUser, model: LenientForm<SetWeightModel>, conn: DbConn) -> Redirect {
    conn.run(move |c| actions::homework::set_weight(&user.into(), &model, c))
        .await
        .unwrap();

    Redirect::to("/")
}
