#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize, Infinite};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Foo {
    Int(i32),
    Float(f32),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Foo>);


fn rcv() -> Result<World, io::Error> {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let connection = SocketAddrV4::new(ip, 9991);

    // Bind the socket
    let socket = try!(UdpSocket::bind(connection));

    let mut buf = [0; 512];

    let (amt, src) = try!(socket.recv_from(&mut buf));

    let decoded: World = deserialize(&buf[..amt]).unwrap();

    Ok(decoded)
}

fn main() {   
    println!("Starting client");

    let (sender, receiver) = channel();

    thread::spawn(move|| {
        loop {
            match rcv() {
                Ok(world) => sender.send(world).unwrap(),
                Err(err) => println!("Error: {:?}", err),
            }
        }
    });

    thread::sleep(Duration::new(10, 0));
    loop {
        let dur = Duration::new(0, 1000);
        match receiver.recv_timeout(dur) {
            Ok(res) => println!("Recieved world {:?}", res),
            Err(err) => {},
        }
    }
}
