#[macro_use(router)]
extern crate router;
extern crate iron;
extern crate urlencoded;

use iron::{status, Response, Request, Iron, IronResult};
use iron::prelude::*;
use urlencoded::UrlEncodedQuery;
use std::process::Command;
use std::{thread, time};

fn respond(ret_status: status::Status, message: &str) -> IronResult<Response> {
    Ok(Response::with((
        ret_status,
        message
    )))
}

fn index(_: &mut Request) -> IronResult<Response> {
    respond(status::Ok, "You found the index!")
}

fn remote_run(remote: &str, actions: &Vec<String>) -> IronResult<Response> {
    let actions: Vec<&str> = actions[0].split(',').collect();
    //let message = format!("Actions: {:?}", actions);
    let mut status = Some(0);
    for action in actions {
        let output = Command::new("irsend")
            .arg("SEND_ONCE")
            .arg(remote)
            .arg(action)
            .output();

        match output {
            Ok(ref out) => status = if !out.status.success() { out.status.code() } else { status },
            Err(_) => status = None
        }

        thread::sleep(time::Duration::from_secs(1));

    }
    let message = match status {
        Some(code) => if code == 0 { "Success" } else { "Failure" },
        None => "Could not find irsend program"
    };
    respond(status::Ok, message)
}

fn remote(req: &mut Request) -> IronResult<Response> {
    let url_ref = req.url.clone();
    let remote_name = url_ref.path()[0];

    let query_ref = match req.url.query() {
        Some(_) => req.get_ref::<UrlEncodedQuery>(),
        None => return respond(
            status::BadRequest,
            "Please add `actions` to the query params"
        )
    };

    let actions = match query_ref {
        Ok(ref hashmap) => hashmap.get("actions"),
        Err(ref e) => return respond(
            status::BadRequest,
            format!("{:?}", e).as_str()
        )
    };

    match actions {
        Some(act) => remote_run(remote_name, act),
        None => respond(
            status::BadRequest,
            format!("No actions specified!").as_str()
        )
    }
}


fn main() {
    let router = router!(
        index: get "/" => index,
        remote: get "/:remote" => remote
    );

    Iron::new(router).http("localhost:3000").unwrap();
}
