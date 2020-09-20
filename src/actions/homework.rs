use crate::models::homework::*;
use crate::pub_imports::*;
use diesel::{prelude::*, PgConnection};
use models::user::User;

pub fn add_homework(
    model: &AddHomeworkModel,
    user: AuthUser,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    let due_date = model.due_date.map(|m| m.0);
    let weekday = if due_date.is_some() {
        None
    } else if let Some(dow) = model.weekday {
        Some(dow)
    } else {
        return Err(HomeworkApiError::UnspecifiedTime);
    };

    let (user_id, class_id) = if model.for_self {
        (Some(user.id), None)
    } else {
        (None, user.class_id)
    };

    let insert = NewHomeworkModel {
        due_date,
        weekday,
        user_id,
        class_id,
        amount: model.amount,
        detail: &model.detail,
    };

    {
        use schema::homework::dsl::*;
        if let Err(e) = diesel::insert_into(homework).values(insert).execute(conn) {
            return Err(HomeworkApiError::DieselError(e));
        }
    }

    Ok(())
}

pub fn get_homework_for_user(
    user: &User,
    conn: &PgConnection,
) -> Result<Vec<DbHomeworkModel>, HomeworkApiError> {
    use schema::homework::dsl::*;
    homework
        .filter(user_id.eq(user.id).or(class_id.eq(user.class_id)))
        .load::<DbHomeworkModel>(conn)
        .map_err(|e| HomeworkApiError::DieselError(e))
}

#[derive(Debug)]
pub enum HomeworkApiError {
    UnspecifiedTime,
    DieselError(diesel::result::Error),
}
impl std::fmt::Display for HomeworkApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}
impl std::error::Error for HomeworkApiError {}
impl From<diesel::result::Error> for HomeworkApiError {
    fn from(e: diesel::result::Error) -> Self {
        HomeworkApiError::DieselError(e)
    }
}
