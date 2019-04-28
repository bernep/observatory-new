// Needed by Rocket
#![feature(proc_macro_hygiene, decl_macro)]

// Ensure all the macros are imported
#[macro_use]
extern crate askama;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel_migrations;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;

use handlers::*;

// Module files
mod guards;
mod handlers;
mod schema;
mod templates;

// Table Modules
mod attend;
mod auth;
mod calendar;
mod groups;
mod news;
mod projects;
mod users;

// Central DB connection
#[database("sqlite_observ")]
pub struct ObservDbConn(diesel::SqliteConnection);

fn main() {

    // Load all the handlers
    use crate::attend::handlers::*;
    use crate::auth::handlers::*;
    use crate::calendar::handlers::*;
    use crate::groups::handlers::*;
    use crate::news::handlers::*;
    use crate::projects::handlers::*;
    use crate::users::handlers::*;

    rocket::ignite()
        // Attach fairings
        .attach(ObservDbConn::fairing())
        .attach(DatabaseCreate)
        .attach(AdminCheck)
        // Register Catchers
        .register(catchers![catch_401, catch_403, catch_404])
        // Mount handlers
        .mount(
            "/",
            routes![
                index,
                staticfile,
                favicon,
                dashboard,
                // Calendar
                calendar,
                calendar_json,
                event,
                editevent,
                editevent_put,
                event_delete,
                newevent,
                newevent_post,
                // Sign Up and Log In
                signup,
                signup_post,
                login,
                login_post,
                logout,
                // Attendance
                attend,
                attend_post,
                // Users
                user,
                user_by_handle,
                users,
                users_json,
                edituser,
                edituser_put,
                user_delete,
                // Projects
                project,
                project_by_handle,
                projects,
                projects_json,
                newproject,
                newproject_post,
                project_delete,
                editproject,
                editproject_put,
                join,
                join_post,
                // Groups
                group,
                groups,
                newgroup,
                newgroup_post,
                group_delete,
                newmeeting_post,
                editgroup,
                editgroup_put,
                // News
                news,
                news_json,
                news_rss,
                news_slides,
                newsstory,
                newnewsstory,
                newnewsstory_post,
                newsstory_delete,
                editnewsstory,
                editnewsstory_put,
            ],
        )
        .launch();
}

// Embed the Migrations into the binary
embed_migrations!();

pub struct DatabaseCreate;

impl Fairing for DatabaseCreate {
    fn info(&self) -> Info {
        Info {
            name: "Create Database if Needed",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, rocket: &Rocket) {

        // Get the database url from the config
        let conn_url = rocket
            .config()
            .get_table("databases")
            .unwrap()
            .get("sqlite_observ")
            .unwrap()
            .get("url")
            .unwrap()
            .as_str()
            .unwrap();

        use diesel::prelude::*;
        let conn = SqliteConnection::establish(conn_url)
            .expect("Failed to connect to database in DatabaseCreate");

        // Run the embedded migrations
        embedded_migrations::run(&conn).expect("Failed to run embedded migrations");
    }
}

// Checks if the Admin user has a password
// and generates one if it doesn't
pub struct AdminCheck;

impl Fairing for AdminCheck {
    fn info(&self) -> Info {
        Info {
            name: "Admin Password Check",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, rocket: &Rocket) {
        // Get the database url from the config
        let conn_url = rocket
            .config()
            .get_table("databases")
            .unwrap()
            .get("sqlite_observ")
            .unwrap()
            .get("url")
            .unwrap()
            .as_str()
            .unwrap();

        use crate::schema::users::dsl::*;
        use crate::users::{NewUser, User};
        use diesel::prelude::*;

        let conn = SqliteConnection::establish(conn_url)
            .expect("Failed to connect to database in AdminCheck");

        let admin: User = users
            .find(0)
            .first(&conn)
            .expect("Failed to get admin from database");

        if admin.password_hash.is_empty() {
            use crate::attend::code::gen_code;
            use crate::auth::crypto::*;

            let pass = gen_code();
            eprintln!(
                "\tADMIN PASSSWORD: {}\n\tCHANGE THIS AS SOON AS POSSIBLE",
                pass
            );

            let psalt = gen_salt();
            let phash = hash_password(pass, &psalt);

            // Needs to be a NewUser for set()
            let nu = NewUser {
                real_name: admin.real_name,
                handle: admin.handle,
                password_hash: phash,
                salt: psalt,
                bio: admin.bio,
                email: admin.email,
                tier: admin.tier,
                active: admin.active,
            };

            use diesel::update;
            update(users.find(0))
                .set(&nu)
                .execute(&conn)
                .expect("Failed to update admin user in database");
        }
    }
}

pub mod models {
    use chrono::NaiveDateTime;
    use std::fmt::Debug;

    pub trait Attendable: Debug {
        fn id(&self) -> i32;
        fn name(&self) -> String;
        fn time(&self) -> NaiveDateTime;
        fn code(&self) -> String;
        fn owner_id(&self) -> i32;
        fn is_event(&self) -> bool;
        fn url(&self) -> String;
    }
}
