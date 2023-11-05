use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, SocketAddr, IpAddr};
use std::time::Duration;
use sysinfo::{System, SystemExt};
use std::{process, str, thread};
use serde::{Deserialize, Serialize};

fn create_socket(client_ip: &str, port: u16) -> UdpSocket {
    let client_address = format!("{}:{}", client_ip, port);
    let socket_addr: SocketAddr = client_address.parse().expect("Failed to parse socket address");
    UdpSocket::bind(socket_addr).expect("Failed to bind socket")
}

fn main() {

    let client_num: u16 = std::env::args()
        .nth(1)
        .expect("didn't specify which port")
        .parse()
        .unwrap();
    
    let client_1 = "127.0.0.4";
    let client_2 = "127.0.0.5";
    let client_3 = "127.0.0.6";

    let clients = vec![client_1, client_2, client_3];

    let client_ip = match client_num {
        1 => client_1,
        2 => client_2,
        3 => client_3,
        _ => panic!("Invalid server number"),
    };

    let listening_port = 5555;
    let sending_port = 6666;

    let server_1_socket = "127.0.0.1:3333";
    let server_2_socket = "127.0.0.2:3333";
    let server_3_socket = "127.0.0.3:3333";

    // client sends to server on port 3333
    // client receives from server on port 9999
    let sending_socket = create_socket(client_ip, 3333);
    let recieving_socket = create_socket(client_ip, 9999);

    println!("Client {} listening on IP address {}", client_num, client_ip);

    for i in 1..20 {
        let mut sending_text = "HELLO ".to_owned() + &i.to_string();
        
        // send to server1
        sending_socket.send_to(sending_text.as_bytes(), &server_1_socket).expect("Failed to send data to server");
        // send to server2
        sending_socket.send_to(sending_text.as_bytes(), &server_2_socket).expect("Failed to send data to server");
        // send to server3
        sending_socket.send_to(sending_text.as_bytes(), &server_3_socket).expect("Failed to send data to server");
    
        // await responses from the leading server
        let mut buffer = [0; 2048];
        let (amt, src) = recieving_socket.recv_from(&mut buffer).expect("Didn't receive data");
        let msg = str::from_utf8(&buffer[..amt]).unwrap();
        print!("{} ", &i.to_string());
        println!("Received: {} from {}", msg, src);
    }
    
}

// fn main() {
//     let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind client socket");

//     let multi_addr1 = "239.0.0.1:9997";
//     let multi_addr2 = "239.0.0.1:9998";
//     let multi_addr3 = "239.0.0.1:9999";
//     let mut buffer = [0; 2048];
    
//     for i in 0..100 {
//         println!("counter: {}", i);

//         let message = "Hello servers: ".to_owned() + &i.to_string();
        
//         socket
//             .send_to(message.as_bytes(), multi_addr1)
//             .expect("Failed to send data to first address");
//         socket
//             .send_to(message.as_bytes(), multi_addr2)
//             .expect("Failed to send data to second address");
//         socket
//             .send_to(message.as_bytes(), multi_addr3)
//             .expect("Failed to send data to third address");

//         let (amt, src) = socket.recv_from(&mut buffer).expect("Didn't receive data");

//         let msg = str::from_utf8(&buffer[..amt]).unwrap();
//         println!("Received: {} FROM {}", msg, src);

//         thread::sleep(Duration::from_secs(2));

//     }
    // Send to the first multicast address

    // let message = "Hello servers: ";

    // socket
    //     .send_to(message.as_bytes(), multi_addr1)
    //     .expect("Failed to send data to first address");

    // // Send to the second multicast address
    // socket
    //     .send_to(message.as_bytes(), multi_addr2)
    //     .expect("Failed to send data to second address");

    // // Send to the third multicast address
    // socket
    //     .send_to(message.as_bytes(), multi_addr3)
    //     .expect("Failed to send data to third address");

    

    // loop{
    // // Receive data from any source
    // let (amt, src) = socket.recv_from(&mut buffer).expect("Didn't receive data");

    // let msg = str::from_utf8(&buffer[..amt]).unwrap();
    // println!("Received: {} FROM {}", msg, src);
    // }

// }
