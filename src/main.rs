use std::{
    fmt::Error,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use simple_file_server::http::{request, response};

fn create_socket() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
}

fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer: [u8; 1024] = [0; 1024];
    stream.read(&mut buffer)?;
    let buf_str = String::from_utf8_lossy(&buffer);
    let request = request::HttpRequest::new(&buf_str)?;
    //let response = request.response()?;
    match request.response() {
        Err(err) => println!("{:?}", err),
        Ok(resp) => {
            //println!("{:?}", resp.response_body);
            stream.write(&resp.response_body)?;
            stream.flush()?;
        }
    }
    Ok(())
}

fn serve(socket: SocketAddr) -> io::Result<()> {
    let listener: TcpListener = TcpListener::bind(socket)?;
    println!(
        "Server is running: http://{}:{}",
        socket.ip(),
        socket.port(),
    );
    let mut counter = 0;
    for stream in listener.incoming() {
        counter += 1;
        match std::thread::spawn(|| handle_client(&mut stream?)).join() {
            Ok(_) =>
                /*println!("Connected stream ... {} ", counter)*/
                {}
            Err(_) => {
                continue;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let socket = create_socket();
    serve(socket)?;
    Ok(())
}
