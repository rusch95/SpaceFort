#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;
use std::thread;
use std::time::Duration;
use bincode::{serialize, deserialize, Infinite};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Foo {
    Int(i32),
    Float(f32),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Foo>);


fn snd() -> Result<(), io::Error> {
    // Define the local connection (to send the data from)
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let connection1 = SocketAddrV4::new(ip, 9990);
    let connection2 = SocketAddrV4::new(ip, 9991);


    let socket = try!(UdpSocket::bind(connection1));
    for i in 0..10 {
        thread::sleep(Duration::new(0, 1000));
        // Send data via the socket
        let world = World(vec![Foo::Int(i), Foo::Float(4.0)]);
        let encoded: Vec<u8> = serialize(&world, Infinite).unwrap();

        try!(socket.send_to(&encoded, connection2));
        println!("World is {:?}", world);
        println!("Server sent serialized {:?}", encoded);
    }

    Ok(())
}

const PORT: u16 = 9090

fn main() {   
    env_logger::init().unwrap();

    info!("Starting server");


}
