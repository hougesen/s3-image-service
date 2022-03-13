use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use dotenv::dotenv;
use rocket::http::ContentType;
use rocket::Data;
use rocket::{self, get, post, routes, Error as RocketError};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use sha2::{Digest, Sha256};
use std::path::Path;

#[get("/")]
async fn index() -> &'static str {
    println!("GET /");
    "hello world"
}

#[post("/upload-image", data = "<data>")]
async fn upload(content_type: &ContentType, data: Data<'_>) -> String {
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
            Ok(_result) => format!("{}/{}", bucket_name, file_name),
            Err(result) => format!("something went wrong {:#?}", result),
        };
    }

    "idk what happened fam".to_string()
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

    rocket.launch().await?;

    Ok(())
}
