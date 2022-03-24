use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use dotenv::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::serde::{json::Json, Serialize};
use rocket::Data;
use rocket::{self, get, post, routes, Error as RocketError};
use rocket::{Request, Response};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Serialize)]
pub struct UploadImageResponse {
    msg: String,
    bucket_name: Option<String>,
    file_name: Option<String>,
    error: Option<String>,
}

#[get("/")]
async fn index() -> &'static str {
    println!("GET /");
    "hello world"
}

#[post("/upload-image", data = "<data>")]
async fn upload(content_type: &ContentType, data: Data<'_>) -> Json<UploadImageResponse> {
    println!("POST /upload-image");
    dotenv().ok();

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .unwrap(),
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .unwrap();

    // Use the get method to preserve file fields from moving out of the MultipartFormData instance in order to delete them automatically when the MultipartFormData instance is being dropped
    let image = multipart_form_data.files.get("image");

    if let Some(file_fields) = image {
        // Because we only put one "photo" field to the allowed_fields, the max length of this file_fields is 1.
        let file_field = &file_fields[0];

        let _content_type = &file_field.content_type;
        let file_name = generate_file_name(&file_field.file_name.clone().unwrap());
        let path = &file_field.path;

        let body = ByteStream::from_path(Path::new(path)).await;

        let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        let bucket_name = dotenv::var("BUCKET_NAME").unwrap();
        let result = client
            .put_object()
            .bucket(&bucket_name)
            .key(&file_name)
            .body(body.unwrap())
            .send()
            .await;

        println!("r {:#?}", result);

        return match result {
            Ok(_result) => Json(UploadImageResponse {
                msg: String::from("Uploaded image successfully"),
                bucket_name: Some(bucket_name),
                file_name: Some(file_name),
                error: None,
            }),
            Err(error) => Json(UploadImageResponse {
                msg: format!("Error uploading file"),
                bucket_name: None,
                file_name: None,
                error: Some(format!("{:#?}", error)),
            }),
        };
    }

    Json(UploadImageResponse {
        msg: String::from("idk what happened fam"),
        bucket_name: None,
        file_name: None,
        error: None,
    })
}

fn generate_file_name(file_name: &str) -> String {
    println!("generate_file_name");
    let s = format!("{}{}", fastrand::i32(..), &file_name.trim());

    let file_ending = s.split('.').collect::<Vec<&str>>().last().copied().unwrap();

    let mut hash = Sha256::new();

    hash.update(&s);

    format!("{:X}.{}", hash.finalize(), file_ending)
}

#[rocket::main]
async fn main() -> Result<(), RocketError> {
    dotenv::dotenv().ok();
    println!("WE LIVE BOYS?");

    let rocket = rocket::build().mount("/", routes![index, upload]);

    rocket.attach(CORS).launch().await?;

    Ok(())
}
