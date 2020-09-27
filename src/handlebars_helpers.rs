use crate::{models::homework::DueDate, pub_imports::*};
use rocket::fairing::Fairing;
use rocket_contrib::templates::{handlebars, Template};

use self::handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

pub fn helpers() -> impl Fairing {
    Template::custom(|engines| {
        let hb = &mut engines.handlebars;
        hb.register_helper("percent", Box::new(percent_helper));
        hb.register_helper("dueDate", Box::new(due_date_helper));
        hb.register_helper("sub", Box::new(sub_helper));
        hb.register_helper("date", Box::new(date_helper));
    })
}

fn percent_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    if let (Some(val), Some(all)) = (h.param(0), h.param(1)) {
        let val = val
            .value()
            .as_f64()
            .unwrap_or_else(|| val.value().as_str().unwrap().parse::<f64>().unwrap());
        let all = all.value().as_f64().unwrap();
        out.write(&(100.0 * val / all).to_string())?;
    }

    Ok(())
}

fn sub_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    if let (Some(v1), Some(v2)) = (h.param(0), h.param(1)) {
        let v1 = v1.value().as_f64().unwrap();
        let v2 = v2.value().as_f64().unwrap();
        out.write(&(v1 - v2).to_string())?;
    }

    Ok(())
}

fn due_date_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let dd: DueDate = serde_json::from_value(h.param(0).unwrap().value().clone()).unwrap();

    match dd {
        DueDate::Date(d) => {
            out.write(&format!("{} tarihi için", d))?;
        }
        DueDate::Repeat(r) => {
            out.write(&format!(
                "Her {}",
                match r {
                    0 => "pazartesiye",
                    1 => "salıya",
                    2 => "çarşambaya",
                    3 => "perşembeye",
                    4 => "cumaya",
                    5 => "cumartesiye",
                    6 => "pazara",
                    _ => unreachable!(),
                }
            ))?;
        }
    }

    Ok(())
}

fn date_helper(
    h: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let day = h.param(0).unwrap().value().as_u64().unwrap() as i32;
    let now = now();
    let day_duration =
        chrono::Duration::from_std(std::time::Duration::from_secs(60 * 60 * 24)).unwrap();

    out.write(&format!("{}", now + day_duration * day))?;
    Ok(())
}
