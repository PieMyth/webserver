# Rust Webserver
This webserver was created as a basic template and learning how to use Actix and Tera in rust to create a website. This has been confirmed to run off of a Raspberry Pi 3, making it a fast and effecient webserver with little power draw. While at the same time being able to dynamically deliver projects onto the webpage by simply just updating the JSON file with more projects!

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

Here are what things you need to install the software and how to install them. The dependencies are provided via Cargo.toml and are mentioned later. This is what is used to develop on, you may be able to use different versions, however your experience may vary.
```
rustc 1.60.0 (7737e0b5c 2022-04-04)
OpenSSL 3.0.2#3 (if you want to support https)
```

### Installing
To build the webserver type:
```
cargo build
```

To run the webserver type:
```
cargo run
```

You should see this output after running:
```
cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.84s
     Running `target\debug\website.exe`
```

You should then be able to either go to:
* http://localhost:8080 - http
* https://localhost:8443 - Do note that this will say there is a problem verifying the certificate as this is a dummy one made locally see [here](https://github.com/actix/examples/tree/master/openssl/) for more details

#### Notes
* If you don't wish to support either http or https, simply comment out or delete the binding in the main.rs file located in the main() function.
* Windows: Make sure to include openssl in windows into an environment variable named `OPENSSL_DIR` more details [here](https://stackoverflow.com/questions/50625283/how-to-install-openssl-in-windows-10)
* Linux: make sure to have `libssl-dev` installed
* Due to the nature of hashmaps, the projects do not show up in a determined order, this is why the vector is sorted according to the rank after the fact

## Built with
* env_logger = "0.9.0" - Debugging logger
* tera = "1.15.0" - Template engine
* actix-web = { version = "4.0.1", features = ["openssl", "rustls"] } - Actix web framework 
* actix-files = "0.6.0" - Static files support for actix web.
* serde = "1.0.114" - Deserialization framework
* serde_json = "1.0.57" - Serialization for JSON files
* futures-util = { version = "0.3.7", default-features = false, features = ["std"] } - Futures for async funcitons
* log = "0.4" - Logger macro
* rustls = "0.20.2" - Support ssl
* rustls-pemfile = "0.2.1" - Pem files (used for the ssl certs)

## Authors

* **William Haugen** - [PieMyth](https://github.com/PieMyth)

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/PieMyth/webserver/blob/master/LICENSE) file for details

## Acknowledgments

Special thanks to:
* Actix examples - [repo/examples](https://github.com/actix/examples)
* Tera examples - [repo/examples](https://github.com/Keats/tera/tree/master/examples)
* Original website template - [FreeWebsiteTemplates](https://freewebsitetemplates.com/preview/ecologicalwebsitetemplate/index.html)
