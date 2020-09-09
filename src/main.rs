#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket_contrib;

use std::collections::HashMap;
use rocket::Rocket;
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::fairing::AdHoc;
use rocket_contrib::{templates::Template, serve::StaticFiles};
use diesel::SqliteConnection;

mod schema;
mod entry;
use crate::entry::Entry;

embed_migrations!();

#[database("sqlite_database")]
pub struct DbConn(SqliteConnection);

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(_) => {
            Err(rocket)
        }
    }
}

#[get("/")]
fn index(conn: DbConn) -> Template {
    let mut context = HashMap::new();
    let entries = Entry::all(&conn);
    context.insert("entries", entries.unwrap());
    Template::render("index", &context)
}

#[post("/", data = "<entry_form>")]
fn add(entry_form: Form<Entry>, conn: DbConn) -> Flash<Redirect> {
    let p = entry_form.into_inner();
    if p.body.is_empty() {
        Flash::error(Redirect::to("/"), "Body cannot be empty")
    } else if let Err(_) = Entry::add(p, &conn) {
        Flash::error(Redirect::to("/"), "Internal Server Error")
    } else {
        Flash::success(Redirect::to("/"), "Created")
    }
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .mount("/", StaticFiles::from("/static"))
        .mount("/", routes![index])
        .mount("/add", routes![add])
        .attach(Template::fairing())
        .launch();
}
