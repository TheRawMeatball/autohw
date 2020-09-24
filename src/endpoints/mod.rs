use crate::pub_imports::*;
use models::user::User;
use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;

mod api;

pub fn routes() -> Vec<rocket::Route> {
    api::routes().add(routes![index, login, add_homework, settings])
}

#[get("/settings")]
fn settings(user: AuthUser, flash: Option<FlashMessage>) -> Template {
    let data = if let Some(msg) = flash {
        let msg = msg.msg();
        json!({
            "title": "Settings",
            "flash": msg,
            "user": user,
        })
    } else {
        json!({
            "title": "Settings",
            "user": user,
        })
    };

    Template::render("settings", &data)
}

#[get("/add")]
fn add_homework(user: AuthUser, flash: Option<FlashMessage>) -> Template {
    let data = if let Some(msg) = flash {
        let msg = msg.msg();
        json!({
            "title": "Add Homework",
            "flash": msg,
            "user": user,
        })
    } else {
        json!({
            "title": "Add Homework",
            "user": user,
        })
    };

    Template::render("add_homework", &data)
}

#[get("/")]
async fn index(user: AuthUser, conn: DbConn) -> Template {
    let u = User::from(user).clone();
    let u2 = u.clone();

    let hw = conn
        .run(move |c| actions::homework::get_homework_for_user(&u, c))
        .await
        .unwrap();

    let schedule = actions::homework::create_schedule(&hw, &[1, 1, 1, 1, 1, /*WEEKEND*/ 1, 1]);

    let data = json!({
        "user": u2,
        "title": "Home",
        "today": schedule[0].1,
        "all": schedule[1..],
        "hw": hw,
    });

    Template::render("index", &data)
}

#[get("/", rank = 2)]
fn login(flash: Option<FlashMessage>) -> Template {
    let data = if let Some(msg) = flash {
        let msg = msg.msg();
        json!({
            "title": "Login",
            "flash": msg,
        })
    } else {
        json!({
            "title": "Login"
        })
    };

    Template::render("login", &data)
}
