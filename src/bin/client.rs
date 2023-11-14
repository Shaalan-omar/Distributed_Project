use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, SocketAddr, IpAddr};
use std::time::Duration;
use sysinfo::{System, SystemExt};
use std::{process, str, thread};
use serde::{Deserialize, Serialize};
use steganography::encoder::*;
use steganography::decoder::*;
use steganography::util::*;
use std::fs::File;
use std::io::{Read, Write};
use image::{DynamicImage, ImageBuffer, Rgba};
use image::GenericImageView;
use base64::{encode, decode};

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

    // load my image and convert it to bytes
    // let my_pic = file_as_dynamic_image("mypic.png".to_string());
    let mut payload = File::open("mypic.png").unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();
    let payload_image_base64 = base64::encode(payload_bytes);
    let payload_image_base64_bytes = payload_image_base64.as_bytes();

    for i in 1..100 {
        
        // send to server1
        sending_socket.send_to(payload_image_base64_bytes, &server_1_socket).expect("Failed to send data to server");
        // send to server2
        sending_socket.send_to(payload_image_base64_bytes, &server_2_socket).expect("Failed to send data to server");
        // send to server3
        sending_socket.send_to(payload_image_base64_bytes, &server_3_socket).expect("Failed to send data to server");
        
        // await responses from the leading server
        let mut buffer = [0; 65535];
        let (amt, src) = recieving_socket.recv_from(&mut buffer).expect("Didn't receive data");
        
        let received_bytes: &[u8] = &buffer[..amt];
        let received_vec = received_bytes.to_vec();
        let mut file = File::create("received.png").unwrap();
        file.write_all(&received_vec).unwrap();

        let encoded_image = file_as_image_buffer("received.png".to_string());
        let dec = Decoder::new(encoded_image);
        let out_buffer = dec.decode_alpha();
        let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| {*b != 0xff_u8}).collect();
        let message = bytes_to_str(clean_buffer.as_slice());
        let decoded_image = base64::decode(message).unwrap();
        let path = format!("decoded_image_{}_client_{}.png", i, client_num);
        let mut file = File::create(path).unwrap();
        file.write_all(&decoded_image);

        println!("Received for i: {} from server: {}", i, src);
        
        // let msg = str::from_utf8(&buffer[..amt]).unwrap();
        // // print!("{} ", &i.to_string());
        // println!("Received: {} from {}", msg, src);
    }
    
}