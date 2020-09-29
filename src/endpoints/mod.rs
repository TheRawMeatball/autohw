use crate::pub_imports::*;
use models::user::User;
use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;

mod api;

pub fn routes() -> Vec<rocket::Route> {
    api::routes().add(routes![index, login, add_homework, settings, past_due])
}

#[get("/settings")]
async fn settings(user: AuthUser, conn: DbConn, flash: Option<FlashMessage<'_, '_>>) -> Template {
    let uid = user.id;
    let weights = conn
        .run(move |c| actions::user::get_user_by_id(uid, c))
        .await
        .unwrap()
        .unwrap()
        .day_weights;

    let data = if let Some(msg) = flash {
        let msg = msg.msg();
        json!({
            "title": "Settings",
            "flash": msg,
            "user": user,
            "weights": weights,
        })
    } else {
        json!({
            "title": "Settings",
            "user": user,
            "weights": weights,
        })
    };

    Template::render("settings", &data)
}

#[get("/add?<amount>&<weight>&<detail>")]
fn add_homework(
    user: AuthUser,
    flash: Option<FlashMessage>,
    amount: Option<i32>,
    weight: Option<i32>,
    detail: Option<String>,
) -> Template {
    let data = json!({
        "title": "Add Homework",
        "flash": flash.map(|f| String::from(f.msg())).unwrap_or(String::from("")),
        "user": user,
        "amount": amount,
        "weight": weight,
        "detail": detail,
        "for_self": weight.map(|_| "checked"),
    });

    Template::render("add_homework", &data)
}

#[get("/")]
async fn index(user: AuthUser, conn: DbConn) -> Template {
    let u = User::from(user).clone();
    let u2 = u.clone();
    let u3 = u.clone();
    let uid = u.id;

    conn.run(move |c| actions::homework::build_progress_table(&u3, c))
        .await
        .unwrap();

    let hw = conn
        .run(move |c| actions::homework::get_homework_for_user(&u, c))
        .await
        .unwrap();

    let weights: Vec<_> = conn
        .run(move |c| actions::user::get_user_by_id(uid, c))
        .await
        .unwrap()
        .unwrap()
        .day_weights
        .into_iter()
        .map(|x| x as i16)
        .collect();

    let schedule = actions::homework::create_schedule(&hw, &weights[0..7]);

    let data = json!({
        "user": u2,
        "title": "Home",
        "today": schedule[0].1,
        "all": schedule[1..],
        "hw": hw,
    });

    Template::render("index", &data)
}

#[get("/past-due")]
async fn past_due(user: AuthUser, conn: DbConn) -> Template {
    let u = User::from(user).clone();
    let u2 = u.clone();

    let hw = conn
        .run(move |c| {
            actions::homework::delete_complete_hw(c);
            actions::homework::get_late_homework(&u, c)
        })
        .await
        .unwrap();

    let data = json!({
        "user": u2,
        "title": "Home",
        "hw": hw,
    });

    Template::render("past_due", &data)
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
