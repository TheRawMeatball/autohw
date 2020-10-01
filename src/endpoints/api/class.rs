use crate::pub_imports::*;

use rocket::{data::ToByteUnit, Data};

pub fn routes() -> Vec<rocket::Route> {
    routes![save_blackboard, get_blackboard].set_root("/class")
}

#[post("/save-blackboard", data = "<json>")]
async fn save_blackboard(user: AuthUser, json: Data, conn: DbConn) {
    let uncompressed = json.open(512.kibibytes()).stream_to_vec().await.unwrap();
    conn.run(move |c| {
        actions::class::set_blackboard(
            user.class_id.unwrap(),
            String::from_utf8(uncompressed).unwrap(),
            c,
        )
    })
    .await
    .unwrap();
}

#[get("/get-blackboard")]
async fn get_blackboard(user: AuthUser, conn: DbConn) -> String {
    let class = conn
        .run(move |c| actions::class::get_class_by_id(user.class_id.unwrap(), c))
        .await
        .unwrap()
        .unwrap();
    class.blackboard
}
