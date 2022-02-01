#[macro_use]
extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_validation::{validator::Validate, Validated};

#[derive(Debug, Deserialize, Serialize, Validate, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct HelloData {
    #[validate(length(min = 3))]
    name: String,
    age: u8,
}

#[get("/hello?<name>&<age>")]
fn hello(name: String, age: u8) -> Json<HelloData> {
    Json(HelloData { name, age })
}

#[get("/validated-hello?<params..>", format = "application/json")]
fn validated_hello(params: Validated<HelloData>) -> Json<HelloData> {
    Json(params.0)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, validated_hello])
}
