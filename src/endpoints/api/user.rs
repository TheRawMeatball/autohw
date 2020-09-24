use crate::pub_imports::*;
use actions::user::{self, UserApiError};
use models::user::*;
use rocket::response::{Flash, Redirect};

pub fn routes() -> Vec<rocket::Route> {
    routes![register, login, logout, change].set_root("/user")
}

#[post("/update", data = "<model>")]
async fn change(
    user: AuthUser,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    model: LenientForm<ChangeFormModel>,
) -> Result<Redirect, Flash<Redirect>> {
    match conn
        .run(move |c| user::change_settings(&*model, &user.into(), &c))
        .await
    {
        Ok(model) => {
            Authenticator::login(cookies, model, conn).await;
            Ok(Redirect::to("/"))
        }
        Err(UserApiError::UserExists) => {
            Err(Flash::error(Redirect::to("/settings"), "Username exists"))
        }
        Err(UserApiError::PasswordTooShort) => Err(Flash::error(
            Redirect::to("/settings"),
            "Password is too short (min 6)",
        )),
        Err(UserApiError::MismatchedPasswords) => Err(Flash::error(
            Redirect::to("/settings"),
            "Passwords do not match",
        )),
        Err(UserApiError::UsernameTooShort) => Err(Flash::error(
            Redirect::to("/settings"),
            "Username too short",
        )),
        Err(_) => Err(Flash::error(Redirect::to("/settings"), "Server error")),
    }
}

#[post("/register", data = "<model>")]
async fn register(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    model: LenientForm<RegisterFormModel>,
) -> Result<Redirect, Flash<Redirect>> {
    match conn.run(move |c| user::add_user(&*model, &c)).await {
        Ok(model) => {
            Authenticator::login(cookies, model, conn).await;
            Ok(Redirect::to("/"))
        }
        Err(UserApiError::UserExists) => Err(Flash::error(Redirect::to("/"), "User exists")),
        Err(UserApiError::PasswordTooShort) => Err(Flash::error(
            Redirect::to("/"),
            "Password is too short (min 6)",
        )),
        Err(UserApiError::MismatchedPasswords) => {
            Err(Flash::error(Redirect::to("/"), "Passwords do not match"))
        }
        Err(UserApiError::UsernameTooShort) => {
            Err(Flash::error(Redirect::to("/"), "Username too short"))
        }
        Err(_) => Err(Flash::error(Redirect::to("/"), "Server error")),
    }
}

#[post("/login", data = "<model>")]
async fn login(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    model: LenientForm<LoginFormModel>,
) -> Result<Redirect, Flash<Redirect>> {
    match conn.run(move |c| user::login(&*model, &c)).await {
        Ok(model) => {
            Authenticator::login(cookies, model, conn).await;
            Ok(Redirect::to("/"))
        }
        Err(UserApiError::UserNotFound) => Err(Flash::error(Redirect::to("/"), "User not found")),
        Err(UserApiError::WrongPassword) => Err(Flash::error(Redirect::to("/"), "Wrong password")),
        Err(_) => Err(Flash::error(Redirect::to("/"), "Server error")),
    }
}

#[get("/logout")]
fn logout(user: AuthUser, cookies: &CookieJar<'_>) -> Redirect {
    user.logout(cookies);
    Redirect::to("/")
}
