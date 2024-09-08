use core::str::FromStr;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Error},
};

use super::response::HttpResponse;

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    pub resource: Resource,
    pub version: Version,
    headers: HttpHeaders,
    request_body: String,
}

impl HttpRequest {
    pub fn response(&self) -> io::Result<HttpResponse> {
        HttpResponse::new(self)
    }
    pub fn new(request: &str) -> io::Result<Self> {
        let method = Method::new(request).map_err(|err| err.to_io_err())?;
        let resource = Resource::new(request).map_err(|err| err.to_io_err())?;
        let version = Version::new(request).map_err(|err| err.to_io_err())?;
        let headers = HttpHeaders::new(request).map_err(|err| err.to_io_err())?;
        let request_body = match method {
            Method::Post => {
                if let Some((_, body)) = request.split_once("\r\n\r\n") {
                    body.to_string()
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        };
        Ok(HttpRequest {
            method,
            resource,
            version,
            headers,
            request_body,
        })
    }
}

#[derive(Debug)]
struct HttpHeaders {
    headers: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn new(request: &str) -> Result<HttpHeaders, ParsingError> {
        let mut http_header = HttpHeaders {
            headers: HashMap::new(),
        };

        let splits = request.split_once("\r\n");

        if let Some((_, header_str)) = splits {
            for line in header_str.split_terminator("\r\n") {
                if line.is_empty() {
                    break;
                }
                if let Some((header, value)) = line.split_once(":") {
                    http_header
                        .headers
                        .insert(header.trim().to_string(), value.trim().to_string());
                }
            }

            Ok(http_header)
        } else {
            Err(ParsingError {
                msg: String::from("parsing headers: Invalid http header"),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    V1_1,
    V2_0,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Version::V1_1 => "HTTP/1.1",
            Version::V2_0 => "HTTP/2",
        };
        write!(f, "{}", content)
    }
}

#[derive(Debug)]
pub struct ParsingError {
    msg: String,
}

impl ParsingError {
    fn to_io_err(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self.msg)
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Version {
    pub fn new(request: &str) -> Result<Self, ParsingError> {
        Version::from_str(request)
    }
}

impl FromStr for Version {
    type Err = ParsingError;

    fn from_str(request: &str) -> Result<Self, Self::Err> {
        let method_version_line = request.split_once("\r\n");
        if let Some((method_or_version_part, _rest)) = method_version_line {
            let parts = method_or_version_part.split_ascii_whitespace();
            let mut result: Result<Version, ParsingError> = Err(ParsingError {
                msg: "".to_string(),
            });
            let mut curr_parse_str: &str = "";
            for line in parts {
                result = match line {
                    "HTTP/1.1" => Ok(Version::V1_1),
                    "HTTP/2" | "HTTP/2.0" => Ok(Version::V2_0),
                    invalid => {
                        curr_parse_str = invalid;
                        result
                    }
                };

                if result.is_ok() {
                    return result;
                }
            }

            if curr_parse_str != "" {
                return Err(ParsingError {
                    msg: format!(
                        "Unknown HTTP version: {} in {}",
                        curr_parse_str, method_or_version_part
                    ),
                });
            }
        }

        Err(ParsingError {
            msg: format!("parsing version: Invalid http header"),
        })
    }
}

#[derive(Debug)]
enum Method {
    Get,
    Post,
    Uninitialized,
}

impl Method {
    pub fn new(request: &str) -> Result<Self, ParsingError> {
        Method::from_str(request)
    }
}

impl FromStr for Method {
    type Err = ParsingError;

    fn from_str(request: &str) -> Result<Self, Self::Err> {
        let method_version_line = request.split_once("\r\n");
        if let Some((method_or_version_part, _rest)) = method_version_line {
            if let Some((method_part, _route_part)) = method_or_version_part.split_once(" ") {
                return match method_part {
                    "POST" => Ok(Method::Post),
                    "GET" => Ok(Method::Get),
                    _ => Ok(Method::Uninitialized),
                };
            }
        }

        Err(ParsingError {
            msg: format!("parsing method: Invalid http header"),
        })
    }
}

#[derive(Debug)]
pub struct Resource {
    pub path: String,
}

impl Resource {
    pub fn new(request: &str) -> Result<Self, ParsingError> {
        Resource::from_str(request)
    }
}

impl FromStr for Resource {
    type Err = ParsingError;

    fn from_str(request: &str) -> Result<Self, Self::Err> {
        let method_version_line = request.split_once("\r\n");
        if let Some((method_or_version_part, _rest)) = method_version_line {
            if let Some((_, route_version_part)) = method_or_version_part.split_once("/") {
                if let Some((route_part, _version_part)) = route_version_part.split_once(" ") {
                    return Ok(Resource {
                        path: ["/", route_part].concat(),
                    });
                } else {
                    return Ok(Resource {
                        path: String::from("/"),
                    });
                }
            }
        }

        Err(ParsingError {
            msg: format!("parsing route: Invalid http header"),
        })
    }
}
