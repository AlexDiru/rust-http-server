use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use itertools::Itertools;
use route_recognizer::{Params, Router};
use threadpool::ThreadPool;

type MyRouter = Router<fn(&Params, &HashMap<String, String>) -> String>;

fn parse_header_line(request_line: &String) -> (String, String) {
    let split = request_line.split(": ");
    let split_vec = split.collect::<Vec<&str>>();
    (split_vec[0].to_string(), split_vec[1].to_string())
}

fn handle_connection(router: &MyRouter, mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut request_line = "".to_string();
    let mut headers: HashMap<String, String> = HashMap::new();

    for (i, line) in http_request.iter().enumerate() {
        println!("Line {}: {}", i, line.clone());
        if i == 0 {
            request_line = line.clone();
        } else if line.len() > 0 {
            let (key, val) = parse_header_line(line);
            headers.insert(key, val);
        }
    }
    //let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("Request line: {}", request_line);

    let req_split = request_line.split(" ");
    let req_split_vec = req_split.collect::<Vec<&str>>();
    let path = req_split_vec[1];

    println!("{}", path);


    let route_fn = router.recognize(path);
    let response = match route_fn {
        Ok(_route_fn) => {
            let endpoint_fn = **_route_fn.handler();
            endpoint_fn(_route_fn.params(), &headers)
        }
        Err(e) => {
            println!("error: {}", e);
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn echo_endpoint(params: &Params, _: &HashMap<String, String>) -> String {
    let params_vec = params.iter().collect_vec();
    let param_val = params_vec[0].1;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        param_val.len(),
        param_val
    )
}

fn ua_endpoint(_: &Params, headers: &HashMap<String, String>) -> String {
    let user_agent_val = headers.get("User-Agent").unwrap();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        user_agent_val.len(),
        user_agent_val
    )
}

fn file_endpoint(params: &Params, headers: &HashMap<String, String>) -> String {

}

fn main() {

    let mut router: MyRouter = Router::new();
    router.add("/echo/:b", echo_endpoint);
    router.add("/", |_, _| "HTTP/1.1 200 OK\r\n\r\n".to_string());
    router.add("/user-agent", ua_endpoint);

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                let router_clone = router.clone();
                pool.execute(move || {
                    handle_connection(&router_clone, _stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
