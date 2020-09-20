use crate::models::user::DbUserModel;
use crate::pub_imports::*;
use rocket::http::Cookie;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthUser {
    pub name: String,
    pub id: i32,
    pub class_name: Option<String>,
    pub class_id: Option<i32>,
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        if let Some(cookie) = req.cookies().get_private("auth") {
            let x = serde_json::from_str::<AuthUser>(
                &std::str::from_utf8(&base64::decode(cookie.value()).unwrap()).unwrap(),
            )
            .unwrap();
            Outcome::Success(x)
        } else {
            Outcome::Forward(())
        }
    }
}

impl AuthUser {
    pub fn logout(self, cookies: &CookieJar<'_>) {
        cookies.remove_private(Cookie::named("auth"));
    }
}

pub(crate) struct Authenticator;

impl Authenticator {
    pub(crate) async fn login(cookies: &CookieJar<'_>, model: DbUserModel, conn: DbConn) {
        cookies.add_private(Cookie::new(
            "auth",
            base64::encode(
                serde_json::to_string(&AuthUser {
                    id: model.id,
                    name: model.name,
                    class_name: if let Some(cid) = model.class_id {
                        Some(
                            conn.run(move |c| {
                                actions::class::get_class_by_id(cid, c)
                                    .unwrap()
                                    .unwrap()
                                    .name
                            })
                            .await,
                        )
                    } else {
                        None
                    },
                    class_id: model.class_id,
                })
                .unwrap(),
            ),
        ))
    }
}
