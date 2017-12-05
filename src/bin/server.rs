use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;

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


fn snd() -> Result<(), io::Error> {
    // Define the local connection (to send the data from)
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let connection = SocketAddrV4::new(ip, 9992);

    // Bind the socket
    let socket = try!(UdpSocket::bind(connection));

    // Define the remote connection (to send the data to)
    let connection2 = SocketAddrV4::new(ip, 9991);

    // Send data via the socket
    let world = World(vec![Foo::Int(5), Foo::Float(4.0)]);
    let encoded: Vec<u8> = serialize(&world, Infinite).unwrap();

    try!(socket.send_to(&encoded, connection2));
    println!("World is {:?}", world);
    println!("Server sent serialized {:?}", encoded);

    Ok(())
}


fn main() {   
    println!("Starting server");
    match snd() {
        Ok(()) => println!("All snd-ing went well"),
        Err(err) => println!("Error: {:?}", err),
    }
}
