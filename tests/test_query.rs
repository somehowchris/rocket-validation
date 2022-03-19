#[macro_use]
extern crate rocket;

use rocket::{
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

#[get("/validated-hello?<params..>")]
fn validated_hello(params: Validated<HelloData>) -> Json<HelloData> {
    Json(params.into_inner())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, validated_hello])
}

use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};

#[test]
pub fn valid_get() {
    let client = Client::tracked(rocket()).unwrap();

    let req = client.get("/hello?name=chris&age=17");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
}

#[test]
pub fn valid_post() {
    let client = Client::tracked(rocket()).unwrap();

    let req = client.get("/validated-hello?name=Chris&age=18");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
}

#[test]
pub fn invalid_short_name() {
    let client = Client::tracked(rocket()).unwrap();

    let req = client.get("/validated-hello?name=CH&age=18");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}

#[test]
pub fn invalid_min_age() {
    let client = Client::tracked(rocket()).unwrap();

    let req = client.get("/validated-hello?name=Chris&age=0");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}

#[test]
pub fn invalid_max_age() {
    let client = Client::tracked(rocket()).unwrap();

    let req = client.get("/validated-hello?name=Chris&age=102");

    let response: LocalResponse = req.dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}
