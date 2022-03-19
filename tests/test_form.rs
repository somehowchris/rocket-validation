#[macro_use]
extern crate rocket;

use rocket::{
    form::Form,
    local::blocking::LocalResponse,
    serde::{json::Json, Deserialize, Serialize},
};
use rocket_validation::{Validate, Validated};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate, FromForm)]
#[serde(crate = "rocket::serde")]
struct HelloData<'a> {
    #[validate(length(min = 3))]
    name: &'a str,
    #[validate(range(min = 1, max = 100))]
    age: u8,
}

#[get("/hello?<name>&<age>")]
fn hello(name: &'_ str, age: u8) -> Json<HelloData> {
    Json(HelloData { name, age })
}

#[post("/hello", data = "<data>")]
fn validated_hello(data: Form<Validated<HelloData>>) -> Json<HelloData> {
    Json(data.0)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, validated_hello])
}

use rocket::{
    http::{ContentType, Header, Status},
    local::blocking::Client,
};

#[test]
pub fn valid_post() {
    let client = Client::tracked(rocket()).unwrap();

    let header = Header::new("content-type", "application/x-www-form-urlencoded");

    let req = client
        .post("/hello")
        .header(header)
        .body("name=Chris&age=18");

    let response: LocalResponse = req.dispatch();

    eprintln!("{:?} {}", response.body(), response.status());
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
}

#[test]
pub fn invalid_short_name() {
    let client = Client::tracked(rocket()).unwrap();

    let header = Header::new("content-type", "application/x-www-form-urlencoded");

    let req = client.post("/hello").header(header).body("name=CH&age=18");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}

#[test]
pub fn invalid_min_age() {
    let client = Client::tracked(rocket()).unwrap();
    let header = Header::new("content-type", "application/x-www-form-urlencoded");

    let req = client
        .post("/hello")
        .header(header)
        .body("name=Chris&age=0");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}

#[test]
pub fn invalid_max_age() {
    let client = Client::tracked(rocket()).unwrap();
    let header = Header::new("content-type", "application/x-www-form-urlencoded");

    let req = client
        .post("/hello")
        .header(header)
        .body("name=Chris&age=102");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}
