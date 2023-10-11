use std::io::{Cursor, Write};

use actix_web::web::{Bytes, Json};
use actix_web::{get, Error, HttpResponse};
use serde::Deserialize;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::analysis::endpoints::get_readonly_endpoints_props;
use crate::models::sc_abi::sc_abi::SCAbi;
use crate::rendering::readonly_endpoint;

#[derive(Deserialize, Debug)]
struct Body {
    sc_abi: SCAbi,
}

#[get("/generate")]
async fn generate(body: Json<Body>) -> Result<HttpResponse, Error> {
    let mut zip_buffer = Vec::new();
    let buffer_cursor = Cursor::new(&mut zip_buffer);
    let mut zip_writer = ZipWriter::new(buffer_cursor);
    let zip_options = FileOptions::default();

    let sc_name = body.sc_abi.name();
    let endpoints = body.sc_abi.endpoints();

    let endpoints_props = get_readonly_endpoints_props(endpoints);

    for (endpoint_name, props) in endpoints_props {
        let rendered_endpoint = readonly_endpoint::render(
            &props.hook_name,
            &endpoint_name,
            &props.inputs,
            &props.outputs,
        );

        zip_writer
            .start_file(format!("/endpoints/{}.ts", &props.file_name), zip_options)
            .unwrap();
        zip_writer.write_all(rendered_endpoint.as_bytes()).unwrap();
    }

    let end_cursor = zip_writer.finish().unwrap();
    let zip_buffer = end_cursor.into_inner();

    let zip_archive_name = format!("{}-dapp.zip", sc_name.to_lowercase());
    let attachment_header = format!("attachment; filename={}", zip_archive_name);

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Disposition", attachment_header))
        .content_type("application/zip")
        .body(Bytes::copy_from_slice(zip_buffer)))
}
