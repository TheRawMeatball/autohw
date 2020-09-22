use std::time;

use crate::models::homework::*;
use crate::pub_imports::*;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::{prelude::*, PgConnection};
use models::{hw_progress::HWProgressModel, user::User};
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
    let source = schema::homework::table.inner_join(schema::hw_progress::table);
    let now = Utc::now().date().naive_local();

    let result = {
        use schema::homework::dsl::*;

        source
            .filter(
                user_id
                    .eq(user.id)
                    .or(class_id.eq(user.class_id))
                    .and(due_date.gt(now)),
            )
            .select((
                id,
                due_date,
                detail,
                amount,
                day_of_week,
                schema::hw_progress::progress,
                schema::hw_progress::delta,
                schema::hw_progress::delta_date,
            ))
            .load::<ProgressHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    for hw in result.iter() {
        if hw.delta_date != Utc::now().date().naive_local() {
            use schema::hw_progress::dsl::*;
            let mut x: HWProgressModel = hw_progress
                .filter(homework_id.eq(hw.id))
                .first(conn)
                .unwrap();
            x.progress += x.delta;
            x.delta = 0;
            x.delta_date = now;
            x.save_changes::<HWProgressModel>(conn).unwrap();
        }
    }

    Ok(result
        .into_iter()
        .map(|model| UserHomework::from(model))
        .collect())
}

#[derive(Clone, Debug)]
struct HwModel {
    hw: UserHomework,
    due: i32,
}

impl HwModel {
    fn left(&self) -> i16 {
        self.hw.amount - self.hw.progress
    }
}

pub fn create_schedule(all: &Vec<UserHomework>) -> Vec<Vec<DailyHomework>> {
    let now = Utc::now().date().naive_local();

    let last_date = all
        .iter()
        .filter_map(|m| match &m.due_date {
            DueDate::Date(d) => Some(d.clone()),
            _ => None,
        })
        .max()
        .unwrap_or(now.succ());

    let mut last_day = (last_date - now).num_days().abs();

    let mut all: Vec<_> = all
        .iter()
        .map(|m| match m.due_date {
            DueDate::Repeat(day_of_week) => {
                let dow = chrono::Weekday::from_i32(day_of_week).unwrap();
                let mut distance = i64::MAX;
                let mut date = NaiveDate::from_weekday_of_month(now.year(), now.month(), dow, 1);

                let mut v = vec![];
                let one_week =
                    Duration::from_std(time::Duration::from_secs(60 * 60 * 24 * 7)).unwrap();

                let mut progress = m.progress;

                loop {
                    if date < now {
                        date = date + one_week;
                        continue;
                    }

                    let day = (last_date - date).num_days().abs();
                    if distance > day {
                        distance = day;

                        if last_day < (date - now).num_days() {
                            last_day = (date - now).num_days();
                        }

                        if progress >= m.amount {
                            progress -= m.amount;
                        } else {
                            let mut m = m.clone();
                            m.detail = format!("{} ({})", m.detail, date);
                            m.progress = progress;
                            progress = 0;
                            v.push(HwModel {
                                hw: m,
                                due: (date - now).num_days() as i32 - 1,
                            });
                        }

                        date = date + one_week;
                    } else {
                        break;
                    }
                }
                v
            }
            DueDate::Date(d) => vec![HwModel {
                hw: m.clone(),
                due: (d - now).num_days() as i32 - 1,
            }],
        })
        .flatten()
        .collect();

    let mut workload: Vec<i16> = vec![0; last_day as usize];

    for hw in all.iter() {
        workload[hw.due as usize] += hw.hw.amount - hw.hw.progress;
    }

    all.sort_by_key(|x| x.due);

    let result =
        create_work_split(workload)
            .iter()
            .fold((all, vec![]), |(mut all, mut v), load| {
                let mut load = *load;
                let mut for_today = vec![];
                while load > 0 {
                    if all[0].left() > load {
                        for_today.push(DailyHomework {
                            hw: all[0].hw.clone(),
                            amount: load as i32,
                        });
                        all[0].hw.progress += load;
                        load = 0; // break;
                    } else {
                        let hw = all.remove(0);
                        let left = hw.left();
                        for_today.push(DailyHomework {
                            hw: hw.hw,
                            amount: left as i32,
                        });
                        load -= left;
                    }
                }
                v.push(for_today);
                (all, v)
            });

    result.1
}

fn create_work_split(workload: Vec<i16>) -> Vec<i16> {
    let last_day = workload.len();
    let mut work_split: Vec<i16> = vec![0; last_day as usize];

    for day in 0..last_day {
        for start in 0..=day {
            let slice = &mut work_split[start as usize..=day as usize];
            let sum = slice.iter().sum::<i16>() as usize + workload[day as usize] as usize;
            let avg = sum as f32 / slice.len() as f32;
            if avg > *slice.iter().max().unwrap() as f32 {
                let added = avg.floor() as i16;

                for load in slice.iter_mut() {
                    *load = added;
                }

                for i in 0..(sum % slice.len()) {
                    slice[i] += 1;
                }

                break;
            }
        }
    }

    work_split
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
