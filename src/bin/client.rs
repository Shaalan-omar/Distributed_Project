use base64::{decode, encode};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::{process, str, thread};
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
use sysinfo::{System, SystemExt};

fn create_socket(client_ip: &str, port: u16) -> UdpSocket {
    let client_address = format!("{}:{}", client_ip, port);
    let socket_addr: SocketAddr = client_address
        .parse()
        .expect("Failed to parse socket address");
    UdpSocket::bind(socket_addr).expect("Failed to bind socket")
}

fn main() {
    let client_num: u16 = std::env::args()
        .nth(1)
        .expect("didn't specify which port")
        .parse()
        .unwrap();

    let client_1 = "172.20.10.10";
    let client_2 = "172.20.10.7";
    let client_3 = "172.20.10.11";

    let clients = vec![client_1, client_2, client_3];

    let client_ip = match client_num {
        1 => client_1,
        2 => client_2,
        3 => client_3,
        _ => panic!("Invalid server number"),
    };

    let listening_port = 5555;
    let sending_port = 6666;

    let server_1_socket = "172.20.10.2:3333";
    let server_2_socket = "172.20.10.9:3333";
    let server_3_socket = "172.20.10.11:3333";

    // client sends to server on port 3333
    // client receives from server on port 9999
    let sending_socket = create_socket(client_ip, 3333);
    let recieving_socket = create_socket(client_ip, 9999);

    println!(
        "Client {} listening on IP address {}",
        client_num, client_ip
    );

    // load my image and convert it to bytes
    let mut payload = File::open("big.png").unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();

    // fragment the image into bytes
    let mut fragmented_image_bytes = Vec::new();
    for chunk in payload_bytes.chunks(1024) {
        fragmented_image_bytes.push(chunk);
    }

    // println!("The size of the image is {}", fragmented_image_bytes.len());

    for i in 1..11 {
        for j in 0..fragmented_image_bytes.len() {
            // send to server1
            sending_socket
                .send_to(fragmented_image_bytes[j], &server_1_socket)
                .expect("Failed to send data to server");
            // send to server2
            sending_socket
                .send_to(fragmented_image_bytes[j], &server_2_socket)
                .expect("Failed to send data to server");
            // send to server3
            sending_socket
                .send_to(fragmented_image_bytes[j], &server_3_socket)
                .expect("Failed to send data to server");
            // println!("Sent chunk {} to all servers", j);
            if j % 20 == 0 && j != 0 {
                // sleep for 1 second
                thread::sleep(Duration::from_millis(10));
            }
        }
        println!("Sent picture number {} to all servers", i);
        sending_socket
            .send_to(b"MINSENDEND", &server_1_socket)
            .expect("Failed to send data to server");
        sending_socket
            .send_to(b"MINSENDEND", &server_2_socket)
            .expect("Failed to send data to server");
        sending_socket
            .send_to(b"MINSENDEND", &server_3_socket)
            .expect("Failed to send data to server");
        println!("Sent end to all servers");

        // // await responses from the leading server
        let mut buffer = [0; 65535];
        let mut src_server;

        let mut image_from_server: Vec<u8> = Vec::new();
        loop {
            let (amt, src) = recieving_socket
                .recv_from(&mut buffer)
                .expect("Didn't receive data");
            let recieved_chunk = &buffer[..amt];
            // println!("Amt: {}", amt);
            if recieved_chunk == b"MINSENDEND" {
                src_server = src.to_string();
                break;
            }
            image_from_server.append(&mut recieved_chunk.to_vec());
        }

        println!("Received encyrption from server: {}", src_server);
        let mut reconstructed_image_bytes = Vec::new();
        for k in 0..image_from_server.len() {
            reconstructed_image_bytes.push(image_from_server[k]);
        }
        let mut file = File::create("final.png").unwrap();
        file.write_all(&reconstructed_image_bytes);

        // decode the file
        let encoded_image = file_as_image_buffer("final.png".to_string());
        let dec = Decoder::new(encoded_image);
        let out_buffer = dec.decode_alpha();
        let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
        let message = bytes_to_str(clean_buffer.as_slice());
        let decoded_image = base64::decode(message).unwrap();
        let path = format!("decoded_image_{}_client_{}.png", i, client_num);
        let mut file = File::create(path).unwrap();
        file.write_all(&decoded_image);
    }
}
