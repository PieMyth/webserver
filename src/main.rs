// Copyright © 2018 William Haugen - Piemyth
// [This work is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
#[allow(unused_imports)]
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tera::Tera;
use serde::{Deserialize, Serialize};
use std::fs::File;
use actix_files as fs;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;
use actix_web_middleware_redirect_https::RedirectHTTPS;

//Structure that follows the json file
#[derive(Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct Project {
    name: String,
    language: Vec<String>,
    description: String,
    implementation: String,
    link: String,
    image: String,
	rank: usize,
}

//Gets all of the projects from the jsonfile in templates/projects.json
//Moves them over into a hashmap, then pushes them onto a Vector of Project structures
//Returns the Vector of Project structures
fn get_projects()-> Vec<Project> {
    let mut projects: Vec<Project>= Vec::new();
        // Create a path to the desired file
    let path = Path::new("./templates/projects.json");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => (),
    }
	
	// Put the json contents into a hashmap
    let p: HashMap<String,Project> = match serde_json::from_str(&s){
        Err(why) => panic!("couldn't convert to json: {}", why),
        Ok(p) => p,
    };

    for proj in p{
        projects.push(proj.1);
    }
	
	projects.sort_by(|a,b| b.rank.cmp(&a.rank));
	projects.reverse();
	
	projects
}

// store tera template in application state
// Gets the projects from the json file and hands them off to tera.
// Renders the page afterwards.
async fn index(
    tmpl: web::Data<tera::Tera>
) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    let projects = get_projects();
    ctx.insert("projects", &projects);
    let s = tmpl.render("index.html", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
	
	// Setup the cert files
	// Comment out if you don't want https
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
	builder.set_private_key_file("./ssl/private.key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("./ssl/domain.cert.pem").unwrap();

	// Creating the Actix HttpServer
    HttpServer::new(|| {
		// Loads all of the html files into tera
        let tera =
            Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")).unwrap();

        App::new()
		// Loads Tera into the webapp
            .data(tera)
		// This will actively divert traffic from http (port 8080) to https (port 8443)
		// Comment out if you intend to only support one way
            .wrap(RedirectHTTPS::with_replacements(&[(":8080".to_owned(), ":8443".to_owned())]))
		// Logger, showing status code of all get requests to the server as well as the time it took
		// Comment out if you don't need it for debugging purposes
            //.wrap(middleware::Logger::default()) // enable logger
		// Tell the server where all of the static assets are
            .service(fs::Files::new("/images", "./templates/images"))
            .service(fs::Files::new("/css", "./templates/css"))
            .service(fs::Files::new("/fonts", "./templates/fonts"))
		// Tell the server where to route traffic to and which html file to use
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/index.html").route(web::get().to(index)))
		// Handle errors that the server may encounter
            .service(web::scope("").wrap(error_handlers()))
    })
	//Open https socket
    .bind_openssl("localhost:8443", builder)?
	//Open http socket
    .bind("localhost:8080")?
    .run()
    .await
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => Response::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
