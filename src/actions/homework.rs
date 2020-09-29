use std::time;

use crate::models::homework::*;
use crate::pub_imports::*;
use chrono::{Datelike, Duration, NaiveDate};
use diesel::{dsl::all, prelude::*, PgConnection};
use models::{hw_progress::*, user::User};
use num_traits::FromPrimitive;

pub fn change_progress(
    user: &User,
    change_model: &ChangeProgressModel,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    use schema::hw_progress::dsl::*;

    let mut model: HWProgressModel = hw_progress
        .filter(user_id.eq(user.id).and(homework_id.eq(change_model.id)))
        .first(conn)
        .optional()
        .map_err(|e| HomeworkApiError::DieselError(e))?
        .unwrap();

    if change_model.use_delta {
        if model.delta_date != now() {
            model.progress += model.delta;
            model.delta = 0;
            model.delta_date = now();
        }
        model.delta += change_model.amount;
    } else {
        model.progress += change_model.amount;
    }

    model
        .save_changes::<HWProgressModel>(conn)
        .map(|_| ())
        .map_err(|e| HomeworkApiError::DieselError(e))
}

pub fn set_weight(
    user: &User,
    change_model: &SetWeightModel,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    use schema::hw_progress::dsl::*;

    diesel::update(hw_progress.filter(user_id.eq(user.id).and(homework_id.eq(change_model.id))))
        .set(weight.eq(change_model.weight))
        .execute(conn)
        .map_err(|e| HomeworkApiError::DieselError(e))
        .map(|_| ())
}

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

    let hw_id = {
        use schema::homework::dsl::*;
        diesel::insert_into(homework)
            .values(insert)
            .get_result::<DbHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
            .id
    };

    let insert = if model.for_self {
        vec![HWProgressModel {
            user_id: user.id,
            delta: 0,
            progress: 0,
            delta_date: now(),
            homework_id: hw_id,
            weight: 1,
            last_repeat_reset: None,
        }]
    } else {
        use schema::users::dsl::*;

        users
            .filter(class_id.eq(user.class_id))
            .select(id)
            .load(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
            .iter()
            .map(|user_id| HWProgressModel {
                user_id: *user_id,
                delta: 0,
                progress: 0,
                delta_date: now(),
                homework_id: hw_id,
                weight: 1,
                last_repeat_reset: None,
            })
            .collect()
    };

    {
        use schema::hw_progress::dsl::*;

        diesel::insert_into(hw_progress)
            .values(insert)
            .execute(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?;
    }

    Ok(())
}

pub fn build_progress_table(user: &User, conn: &PgConnection) -> Result<(), HomeworkApiError> {
    let now = now();

    let user_hw = {
        use schema::homework::dsl::*;

        homework
            .filter((user_id.eq(user.id).or(class_id.eq(user.class_id))).and(due_date.gt(now)))
            .load::<DbHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    let user_progress = {
        use schema::hw_progress::dsl::*;

        hw_progress
            .filter(user_id.eq(user.id))
            .load::<HWProgressModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    let inserts: Vec<_> = user_hw
        .iter()
        .filter_map(|hw: &DbHomeworkModel| {
            if user_progress
                .iter()
                .find(|&x| x.user_id == user.id && x.homework_id == hw.id)
                .is_some()
            {
                None
            } else {
                Some(HWProgressModel {
                    delta: 0,
                    delta_date: now,
                    homework_id: hw.id,
                    progress: 0,
                    user_id: user.id,
                    weight: 1,
                    last_repeat_reset: None,
                })
            }
        })
        .collect();

    {
        use schema::hw_progress::dsl::*;

        diesel::insert_into(hw_progress)
            .values(inserts)
            .execute(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?;
    }

    Ok(())
}

pub fn get_homework_for_user(
    user: &User,
    conn: &PgConnection,
) -> Result<Vec<UserHomework>, HomeworkApiError> {
    let source = schema::homework::table.inner_join(schema::hw_progress::table);
    let now = now();

    let result = {
        use schema::homework::dsl::*;
        source
            .filter(
                (user_id.eq(user.id).or(class_id.eq(user.class_id)))
                    .and(due_date.gt(now).or(due_date.is_null()))
                    .and(schema::hw_progress::user_id.eq(user.id)),
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
                schema::hw_progress::weight,
                schema::hw_progress::last_repeat_reset,
            ))
            .load::<ProgressHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    for hw in result.iter() {
        use schema::hw_progress::dsl::*;
        let mut x: HWProgressModel = hw_progress
            .filter(homework_id.eq(hw.id))
            .filter(user_id.eq(user.id))
            .first(conn)
            .unwrap();

        if hw.delta_date != now {
            x.progress += x.delta;
            x.delta = 0;
            x.delta_date = now;
        }

        if let Some(dow) = hw.day_of_week {
            if now.weekday().num_days_from_monday() == dow as u32
                && hw.last_repeat_reset != Some(now)
            {
                x.last_repeat_reset = Some(now);
                if hw.progress - hw.amount >= 0 {
                    x.progress -= hw.amount;
                }
            }
        }

        x.save_changes::<HWProgressModel>(conn).unwrap();
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

    fn eff_left(&self) -> i16 {
        self.left() * self.hw.weight as i16
    }
}

pub fn create_schedule(all: &Vec<UserHomework>, weights: &[i16]) -> Vec<(i32, Vec<DailyHomework>)> {
    let now = now();

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
                    if date <= now {
                        date += one_week;
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

    let one_day = Duration::from_std(time::Duration::from_secs(60 * 60 * 24)).unwrap();

    let mut workload = vec![0i16; last_day as usize];

    for hw in all.iter() {
        workload[hw.due as usize] += (hw.hw.amount - hw.hw.progress) * hw.hw.weight as i16;
    }

    let mut work_split: Vec<_> = (0..last_day as i32)
        .into_iter()
        .map(|day| {
            (
                weights[(now + one_day * day).weekday().num_days_from_monday() as usize] as i32,
                workload[day as usize] as i32,
            )
        })
        .collect();

    all.sort_by_key(|x| x.due);

    distribute(&mut work_split);

    work_split
        .iter()
        .map(|&(w, l)| (w as i16, l as i16))
        .enumerate()
        .fold(
            (all, vec![], 0),
            |(mut all, mut v, mut next_borrow), (day, (_, mut load))| {
                let mut for_today = vec![];

                if next_borrow > load {
                    return (all, v, next_borrow - load);
                } else {
                    load -= next_borrow;
                }

                while load > 0 {
                    if day != 0 {
                        all[0].hw.delta = 0;
                    }

                    if all[0].eff_left() > load {
                        let div = load / all[0].hw.weight as i16;
                        let rem = load % all[0].hw.weight as i16;
                        let work = (div + rem.min(1)) as i32;
                        for_today.push(DailyHomework {
                            hw: all[0].hw.clone(),
                            amount: work,
                        });
                        all[0].hw.progress += work as i16;
                        next_borrow = rem.min(1) * (all[0].hw.weight - rem as i32) as i16;
                        load = 0; // break;
                    } else {
                        let hw = all.remove(0);
                        let left = hw.left();
                        let eleft = hw.eff_left();
                        for_today.push(DailyHomework {
                            hw: hw.hw,
                            amount: left as i32,
                        });
                        load -= eleft;
                    }
                }
                v.push((day as i32, for_today));
                (all, v, next_borrow)
            },
        )
        .1
        .into_iter()
        .map(|(due, mut vec)| {
            vec.sort_by_key(|hw| hw.hw.db_id);
            (due, vec)
        })
        .collect()
}

/// In the tuple, first element is weight for x, and second is the amount of homework due x+1.
/// Once the function returns, the weights will not be altered, but, the second will be the amount
/// of homework that should be done on day x.
fn distribute(work: &mut [(i32, i32)]) {
    for day in 0..work.len() {
        let day_load = work[day].1;
        work[day].1 = 0;
        'day_loop: for start in 0..=day {
            let slice = &mut work[start..=day];
            for partial in 0..slice[0].0 {
                let ws = slice[0];
                let partial_load =
                    (ws.0 - partial) * (ws.1 / ws.0) + ((ws.1 % ws.0) - partial).max(0);

                let load = day_load + partial_load + slice[1..].iter().map(|x| x.1).sum::<i32>();
                let effective_len = slice.iter().map(|x| x.0).sum::<i32>() - partial;
                let max = slice[1..]
                    .iter()
                    .map(|&(w, l)| {
                        if w == 0 {
                            0
                        } else {
                            l / w + i32::min(l % w, 1)
                        }
                    })
                    .max()
                    .unwrap_or(0)
                    .max((ws.1 / ws.0) + ((ws.1 % ws.0) - partial).max(0));

                if load / effective_len >= max {
                    slice[0].1 += (load / effective_len) * (slice[0].0 - partial) - partial_load;

                    for x in slice[1..].iter_mut() {
                        x.1 = x.0 * (load / effective_len);
                    }

                    let mut excess = load % effective_len;

                    for x in slice.iter_mut().take((load % effective_len) as usize) {
                        x.1 += x.0.min(excess);
                        excess -= x.0.min(excess);

                        if excess == 0 {
                            break;
                        }
                    }

                    break 'day_loop;
                }
            }
        }
    }
}

pub fn get_late_homework(
    user: &User,
    conn: &PgConnection,
) -> Result<Vec<UserHomework>, HomeworkApiError> {
    let source = schema::homework::table.inner_join(schema::hw_progress::table);
    let now = now();

    let result = {
        use schema::homework::dsl::*;

        source
            .filter(
                (user_id.eq(user.id).or(class_id.eq(user.class_id)))
                    .and(due_date.le(now))
                    .and(due_date.is_not_null())
                    .and(schema::hw_progress::user_id.eq(user.id)),
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
                schema::hw_progress::weight,
                schema::hw_progress::last_repeat_reset,
            ))
            .load::<ProgressHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    for hw in result.iter() {
        use schema::hw_progress::dsl::*;
        let mut x: HWProgressModel = hw_progress
            .filter(homework_id.eq(hw.id))
            .filter(user_id.eq(user.id))
            .first(conn)
            .unwrap();

        if hw.delta_date != now {
            x.progress += x.delta;
            x.delta = 0;
            x.delta_date = now;
        }

        x.save_changes::<HWProgressModel>(conn).unwrap();
    }

    Ok(result.into_iter().map(|phm| phm.into()).collect())
}

pub fn delete_old_hw_for_user(uid: i32, old: i32, conn: &PgConnection) {
    use schema::hw_progress::dsl::*;

    diesel::delete(hw_progress.filter(homework_id.eq(old).and(user_id.eq(uid))))
        .execute(conn)
        .unwrap();

    {
        use schema::homework::dsl::*;

        diesel::delete(homework.filter(id.ne(all(hw_progress.select(homework_id)))))
            .execute(conn)
            .unwrap();
    }
}

pub fn delete_complete_hw(conn: &PgConnection) {
    let now = now();

    let query = format!(
        r#"
        delete from "hw_progress" where ("user_id", "homework_id") in (
            select "hw_progress"."user_id", "hw_progress"."homework_id" FROM hw_progress
            inner join "homework" on "homework"."id" = "hw_progress"."homework_id"
            where ("homework"."amount" = "hw_progress"."progress") and ("homework"."due_date" <= '{}')
        );
        "#, now);

    diesel::sql_query(query)
        .execute(conn)
        .unwrap();

    {
        use schema::homework::dsl::*;
        use schema::hw_progress::dsl::*;

        diesel::delete(homework.filter(id.ne(all(hw_progress.select(homework_id)))))
            .execute(conn)
            .unwrap();
    }
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
