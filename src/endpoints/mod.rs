use crate::pub_imports::*;
use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;
use models::user::User;

mod api;

pub fn routes() -> Vec<rocket::Route> {
    api::routes().add(routes![index, login])
}

#[get("/")]
async fn index(user: AuthUser, conn: DbConn) -> Template {
    let u = User::from(user).clone();
    let u2 = u.clone();
    let hw = conn.run(move |c| actions::homework::get_homework_for_user(&u, c)).await.unwrap();
    
    let data = json!({
        "user": u2,
        "title": "Home",
        "homework": hw,
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
