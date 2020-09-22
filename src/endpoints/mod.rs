use crate::pub_imports::*;
use chrono::NaiveDate;
use models::{homework::UserHomework, user::User};
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

    let mock_data = vec![
        UserHomework {
            amount: 5,
            db_id: -1,
            detail: "9-24".into(),
            due_date: models::homework::DueDate::Date(NaiveDate::from_ymd(2020, 9, 24)),
            progress: 2,
            delta: 2,
        },
        UserHomework {
            amount: 12,
            db_id: -1,
            detail: "9-30".into(),
            due_date: models::homework::DueDate::Date(NaiveDate::from_ymd(2020, 9, 30)),
            progress: 2,
            delta: 0,
        },
        UserHomework {
            amount: 13,
            db_id: -1,
            detail: "9-28".into(),
            due_date: models::homework::DueDate::Date(NaiveDate::from_ymd(2020, 9, 28)),
            progress: 4,
            delta: 0,
        },
        UserHomework {
            amount: 17,
            db_id: -1,
            detail: "r4".into(),
            due_date: models::homework::DueDate::Repeat(4),
            progress: 22,
            delta: 0,
        },
        UserHomework {
            amount: 2,
            db_id: -1,
            detail: "r3".into(),
            due_date: models::homework::DueDate::Repeat(3),
            progress: 0,
            delta: 0,
        },
        UserHomework {
            amount: 20,
            db_id: -1,
            detail: "10-1".into(),
            due_date: models::homework::DueDate::Date(NaiveDate::from_ymd(2020, 10, 1)),
            progress: 4,
            delta: 0,
        },
    ];

    let hw = mock_data; //conn.run(move |c| actions::homework::get_homework_for_user(&u, c)).await.unwrap();
    let schedule = actions::homework::create_schedule(&hw);

    let data = json!({
        "user": u2,
        "title": "Home",
        "today": schedule[0],
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
