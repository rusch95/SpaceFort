#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize, Infinite};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Foo {
    Int(i32),
    Float(f32),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Foo>);

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;


fn rcv() -> Result<(), io::Error> {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let connection = SocketAddrV4::new(ip, 9991);

    // Bind the socket
    let socket = try!(UdpSocket::bind(connection));

    let mut buf = [0; 512];

    let (amt, src) = try!(socket.recv_from(&mut buf));

    let decoded: World = deserialize(&buf[..amt]).unwrap();
    println!("Client recieved {:?}", &buf[..amt]);
    println!("Client says world is {:?}", decoded);

    Ok(())
}

fn main() {   
    println!("Starting client");

    match rcv() {
        Ok(()) => println!("All rcv-ing went well"),
        Err(err) => println!("Error: {:?}", err),
    }
}
