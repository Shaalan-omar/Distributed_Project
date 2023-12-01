use base64::{decode, encode};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug)]
// struct that contains image fragment and request type
struct ImageFragment {
    fragment: Vec<u8>,
    request_type: u8,
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

    // request type to server:
    // 1. send image
    // 2. ask for directory of service
    let request_type_image = 1;
    let request_type_directory: u8 = 2;

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

    for i in 1..6 {
        if i % 2 == 0 {
            // send for directory of service every other time
            let directory_request = ImageFragment {
                fragment: Vec::new(),
                request_type: request_type_directory,
            };
            let encoded = serde_json::to_string(&directory_request).unwrap();
            sending_socket
                .send_to(encoded.as_bytes(), &server_1_socket)
                .expect("Failed to send data to server");
            sending_socket
                .send_to(encoded.as_bytes(), &server_2_socket)
                .expect("Failed to send data to server");
            sending_socket
                .send_to(encoded.as_bytes(), &server_3_socket)
                .expect("Failed to send data to server");
            println!("Sent directory request to all servers");
        } else {
            // else send the image to all servers.
            for j in 0..fragmented_image_bytes.len() {
                // send the struct to the server
                let image_fragment = ImageFragment {
                    fragment: fragmented_image_bytes[j].to_vec(),
                    request_type: request_type_image,
                };

                let encoded = serde_json::to_string(&image_fragment).unwrap();

                // send to server1
                sending_socket
                    .send_to(encoded.as_bytes(), &server_1_socket)
                    .expect("Failed to send data to server");
                // send to server2
                sending_socket
                    .send_to(encoded.as_bytes(), &server_2_socket)
                    .expect("Failed to send data to server");
                // send to server3
                sending_socket
                    .send_to(encoded.as_bytes(), &server_3_socket)
                    .expect("Failed to send data to server");

                if j % 20 == 0 && j != 0 {
                    // sleep for 1 second
                    thread::sleep(Duration::from_millis(10));
                }
            }
            println!("Sent picture number {} to all servers", i);

            // send end to all servers
            let end_message = "MINSENDEND";
            let final_message = ImageFragment {
                fragment: end_message.as_bytes().to_vec(),
                request_type: request_type_image,
            };
            let encoded = serde_json::to_string(&final_message).unwrap();

            sending_socket
                .send_to(encoded.as_bytes(), &server_1_socket)
                .expect("Failed to send data to server");
            sending_socket
                .send_to(encoded.as_bytes(), &server_2_socket)
                .expect("Failed to send data to server");
            sending_socket
                .send_to(encoded.as_bytes(), &server_3_socket)
                .expect("Failed to send data to server");
            println!("Sent end to all servers");
        }

        // // await responses from the leading server
        let mut buffer = [0; 65535];
        let mut src_server;

        let mut image_from_server: Vec<u8> = Vec::new();
        let mut isimage = true;
        loop {
            // recieve image fragments from server, if it is a directory, print it, else, append to image_from_server
            let (amt, src) = recieving_socket
                .recv_from(&mut buffer)
                .expect("Didn't receive data");

            let msg = str::from_utf8(&buffer[..amt]).unwrap();
            let image_fragment: ImageFragment = serde_json::from_str(msg).unwrap();
            let recieved_chunk = image_fragment.fragment;
            let request_type = image_fragment.request_type;
            src_server = src.to_string();

            if request_type == request_type_directory {
                println!("Received directory from server: {}", src);
                println!("{}", str::from_utf8(&recieved_chunk).unwrap());
                isimage = false;
                // HERE WE CAN DO SOMETHING WITH THE DIRECTORY
                break;
            } else if request_type == request_type_image {
                if recieved_chunk == b"MINSENDEND" {
                    break;
                }
                image_from_server.append(&mut recieved_chunk.to_vec());
            }
        }

        // if we recieved a directory, continue and dont decode the image
        if !isimage {
            isimage = true;
            continue;
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
