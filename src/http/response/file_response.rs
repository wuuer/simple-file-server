use once_cell::sync::Lazy;
use std::clone;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::AcceptRanges;
use super::HttpResponse;
use super::ResponseStatus;
use crate::http::request;
use crate::http::request::Version;

static RESOURCE_FOLDER: &str = "wwwroot";

static SERVER_ROOT_PATH: Lazy<io::Result<PathBuf>> =
    Lazy::new(|| Ok(std::env::current_dir()?.join(RESOURCE_FOLDER)));

static DIR_BEGIN_HTML: &str = r#"
<!DOCTYPE html> 
<html> 
<head> 
    <meta charset="utf-8"> 
</head> 
<body>"#;

static DIR_END_HTML: &str = r#"
    </body>
    </html>"#;

fn response(
    version: &Version,
    status: &ResponseStatus,
    accept_ranges: &AcceptRanges,
    content_type: &str,
    content_length: usize,
    content: Vec<u8>,
) -> Vec<u8> {
    let mut response_body_arr = format!(
        "{} {}\n{}\ncontent-type: {}\ncontent-length: {}\r\n\r\n",
        version, status, accept_ranges, content_type, content_length
    )
    .into_bytes();

    response_body_arr.extend_from_slice(&content);

    response_body_arr
}

fn file_response(resource_file_path: String) -> io::Result<HttpResponse> {
    let version = request::Version::V1_1;
    let status: ResponseStatus = ResponseStatus::OK;
    let accept_ranges: AcceptRanges = AcceptRanges::Bytes;
    let content = std::fs::read(&resource_file_path)?;
    let content_type;
    if let Some(kind) = infer::get(content.as_ref()) {
        content_type = kind.mime_type().to_string();
    } else {
        content_type = "text/plain".to_string()
    }

    let content_length = content.len();
    let response_body = response(
        &version,
        &status,
        &accept_ranges,
        &content_type,
        content_length,
        content,
    );

    Ok(HttpResponse {
        version,
        status,
        content_length,
        content_type,
        accept_ranges,
        response_body,
        current_path: resource_file_path,
    })
}

fn dir_response(resource_file_path: String, resource: &str) -> io::Result<HttpResponse> {
    let version = request::Version::V1_1;
    let status = ResponseStatus::OK;
    let accept_ranges: AcceptRanges = AcceptRanges::Bytes;
    let entries = fs::read_dir(&resource_file_path)?;
    let header = format!(
        "<h1>Currently in {}</h1>\
        <a href=\"../\">Go back up a directory</a><hr/>
        ",
        resource_file_path
    );
    let mut file_list_body = String::new();
    for entry in entries {
        if let Ok(e) = entry {
            if let Some(file_name) = e.file_name().to_str() {
                file_list_body.push_str(
                    &[
                        "<a href='",
                        &url_escape::encode_path(resource),
                        "/",
                        &url_escape::encode_path(file_name),
                        "'>",
                        file_name,
                        "</a></br>",
                    ]
                    .concat(),
                );
            }
        }
    }
    let content = format!(
        r#"
           {DIR_BEGIN_HTML}
           {header}
           {file_list_body}
           {DIR_END_HTML}
        "#
    );
    let content_length = content.len();
    let content_type = "text/html".to_string();
    let response_body = response(
        &version,
        &status,
        &accept_ranges,
        &content_type,
        content_length,
        content.into_bytes(),
    );

    Ok(HttpResponse {
        version,
        status,
        content_type,
        content_length,
        accept_ranges,
        response_body,
        current_path: resource_file_path,
    })
}

fn four_o_four_response(resource_file_path: String) -> io::Result<HttpResponse> {
    let version = request::Version::V1_1;
    let status: ResponseStatus = ResponseStatus::NotFound;
    let mut content_length: usize = 0;
    let accept_ranges: AcceptRanges = AcceptRanges::None;
    let content = "
                <html>
                <body>
                <h1>404 NOT FOUND</h1>
                </body>
                </html>";
    content_length = content.len();
    let content_type = "text/html".to_string();
    let response_body = response(
        &version,
        &status,
        &accept_ranges,
        &content_type,
        content_length,
        content.as_bytes().to_vec(),
    );

    Ok(HttpResponse {
        version,
        status,
        content_type,
        content_length,
        accept_ranges,
        response_body,
        current_path: resource_file_path,
    })
}

fn to_io_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "invalid resource")
}

pub fn handle_file_response(resource: &str) -> io::Result<HttpResponse> {
    let server_root = (*SERVER_ROOT_PATH)
        .as_ref()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let binding = url_escape::decode(resource);
    let decoded_resource = binding.as_ref();
    let resource_file_path_buf = server_root.join(decoded_resource);
    let mut resource_file_path = resource_file_path_buf.to_str().ok_or_else(to_io_error)?;

    if resource_file_path_buf.exists() {
        let rootcwd_len = server_root.canonicalize()?.components().count();
        let rescwd_len = resource_file_path_buf.canonicalize()?.components().count();
        // prevent from backtracking
        if rescwd_len < rootcwd_len {
            // Backtracking dected !! Move back to the server root.
            resource_file_path = server_root.to_str().ok_or_else(to_io_error)?;
        }

        if resource_file_path_buf.is_file() {
            return file_response(resource_file_path.to_string());
        } else if resource_file_path_buf.is_dir() {
            return dir_response(resource_file_path.to_string(), resource);
        }
    }
    return four_o_four_response(resource_file_path.to_string());
}
