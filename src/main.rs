// Copyright © 2018 William Haugen - Piemyth
// [This work is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use std::{fs::File, io::{BufReader, Read}, path::Path, collections::HashMap};

use actix_web::{error, dev::{ServiceResponse, Service}, get, web, Error, http, App, HttpResponse, HttpServer, Result};
use actix_web::{body::BoxBody, http::StatusCode};
use actix_files::Files;
#[allow(unused_imports)] // Added in case the logger is not used
use actix_web::middleware::{Logger, ErrorHandlerResponse, ErrorHandlers};
use futures_util::future::{self, Either, FutureExt};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tera::Tera;
use serde::{Deserialize, Serialize};

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
	
	projects.sort_by(|a,b| a.rank.cmp(&b.rank));
	
	projects
}

// store tera template in application state
// Gets the projects from the json file and hands them off to tera.
// Renders the page afterwards.
#[get("/")]
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Setup the cert files
	// Comment out if you don't want https
    let cert_file = &mut BufReader::new(File::open("./ssl/domain.cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("./ssl/private.key.pem").unwrap());

    let cert_chain: Vec<Certificate> = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .unwrap();    

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
            // Loads all of the html files into tera
            let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")).unwrap();

        App::new()
            // Loads Tera into the webapp
            .app_data(web::Data::new(tera))
            // Logger, showing status code of all get requests to the server as well as the time it took
		    // Comment out if you don't need it for debugging purposes
            // .wrap(Logger::default()) // enable logger
            // Tell the server where all of the static assets are
            .service(Files::new("/images", "./templates/images"))
            .service(Files::new("/css", "./templates/css"))
            .service(Files::new("/fonts", "./templates/fonts"))
            .service(index)
            .service(web::scope("").wrap(error_handlers()))
            .wrap_fn(|sreq, srv| {
                let host = sreq.connection_info().host().to_owned();
                let uri = sreq.uri().to_owned();
                let url = format!("https://{}{}", host, uri);

                // If the scheme is "https" then it will let other services below this wrap_fn
                // handle the request and if it's "http" then a response with redirect status code
                // will be sent whose "location" header will be same as before, with just "http"
                // changed to "https"
                //
                if sreq.connection_info().scheme() == "https" {
                    Either::Left(srv.call(sreq).map(|res| res))
                } else {
                    return Either::Right(future::ready(Ok(sreq.into_response(
                        HttpResponse::MovedPermanently()
                            .append_header((http::header::LOCATION, url))
                            .finish(),
                    ))));
                }
            })
    })
    .bind(("localhost", 80))? // HTTP port
    .bind_rustls(("localhost", 443), config)? // HTTPS port
    .run()
    .await
}


fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>>  {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
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
                Ok(body) => HttpResponse::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}