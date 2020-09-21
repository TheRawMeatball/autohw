use std::time;

use crate::models::homework::*;
use crate::pub_imports::*;
use chrono::{Datelike, Duration, Utc};
use diesel::{prelude::*, PgConnection};
use models::user::User;
use num_traits::FromPrimitive;

pub fn add_homework(
    model: &AddHomeworkModel,
    user: AuthUser,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    let due_date = model.due_date.map(|m| m.0);
    let day_of_week = if due_date.is_some() {
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
        day_of_week,
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
) -> Result<Vec<UserHomework>, HomeworkApiError> {
    use schema::homework::dsl::*;

    let source = homework.inner_join(schema::hw_progress::table);

    source
        .filter(user_id.eq(user.id).or(class_id.eq(user.class_id)))
        .select((
            id,
            due_date,
            detail,
            amount,
            day_of_week,
            schema::hw_progress::progress,
        ))
        .load::<ProgressHomeworkModel>(conn)
        .map_err(|e| HomeworkApiError::DieselError(e))
        .map(|models| {
            models
                .into_iter()
                .map(|model| UserHomework::from(model))
                .collect()
        })
}

struct HwModel {
    hw: UserHomework,
    due: i32,
}

pub fn create_schedule(all: Vec<UserHomework>, weights: [i32; 7]) -> Vec<Vec<DailyHomework>> {
    let now = Utc::now().date().naive_local();

    let (fixed, repeat): (Vec<_>, Vec<_>) = all
        .into_iter()
        .map(|m| match m.due_date {
            DueDate::Date(_) => (Some(m), None),
            DueDate::Repeat(_) => (None, Some(m)),
        })
        .unzip();

    let last_date = fixed
        .iter()
        .filter_map(|m| {
            m.as_ref().map(|model| match &model.due_date {
                DueDate::Date(d) => d.clone(),
                _ => unreachable!(),
            })
        })
        .max()
        .unwrap_or(now.succ());

    let last_day = (last_date - now).num_days() as i32;

    let fixed = fixed.into_iter().filter_map(|x| x);
    let repeat = repeat.into_iter().filter_map(|x| x);

    let all : Vec<HwModel> = repeat
        .flat_map(|model| match model.due_date {
            DueDate::Repeat(day_of_week) => {
                let dow = chrono::Weekday::from_i32(day_of_week).unwrap();
                let mut dist = i64::MAX;
                let mut date =
                    chrono::NaiveDate::from_weekday_of_month(now.year(), now.month(), dow, 1);

                let mut v = vec![];

                loop {
                    if date < now {
                        date = date
                            + Duration::from_std(time::Duration::from_secs(60 * 60 * 24 * 7))
                                .unwrap();
                        continue;
                    }

                    if dist > (last_date - date).num_days().abs() {
                        dist = (last_date - date).num_days().abs();

                        v.push(HwModel {
                            hw: model.clone(),
                            due: (last_date - date).num_days() as i32,
                        });

                        date = date
                            + Duration::from_std(time::Duration::from_secs(60 * 60 * 24 * 7))
                                .unwrap();
                    } else {
                        break;
                    }
                }
                v
            }
            _ => unreachable!(),
        })
        .chain(fixed.map(|model| match model.due_date {
            DueDate::Date(d) => HwModel {
                hw: model,
                due: (last_date - d).num_days() as i32,
            },
            _ => unreachable!(),
        }))
        .collect();

    panic!();
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
