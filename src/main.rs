#[macro_use] extern crate rocket;

use rocket::fs::FileServer;

#[get("/")]
async fn index() -> &'static str {
    "Hello, bitches!"
}

// This is the main function
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/static", FileServer::from("static"))
}