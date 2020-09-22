use crate::pub_imports::*;
use models::user::User;
use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;

mod api;

pub fn routes() -> Vec<rocket::Route> {
    api::routes().add(routes![index, login])
}

#[get("/")]
async fn index(user: AuthUser, conn: DbConn) -> Template {
    let u = User::from(user).clone();
    let u2 = u.clone();
    let hw = conn
        .run(move |c| actions::homework::get_homework_for_user(&u, c))
        .await
        .unwrap();
    //let today_hw = actions::homework::create_schedule(uhw, [1;7]);

    let data = json!({
        "user": u2,
        "title": "Home",
        "homework": [
            {
                "amount": 3,
                "details": {
                    "detail":"hi",
                    "progress":5,
                    "amount":20,
                    "due_date":{
                        "Date":"2020-09-30",
                    },
                },
            },
            {
                "amount": 5,
                "details": {
                    "detail":"hi",
                    "progress":5,
                    "amount":13,
                    "due_date":{
                        "Date":"2020-09-30",
                    },
                },
            },
            {
                "amount": 9,
                "details": {
                    "detail":"hi",
                    "progress":2,
                    "amount":16,
                    "due_date":{
                        "Date":"2020-09-30",
                    },
                },
            },
        ],
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
