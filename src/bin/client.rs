use base64::{decode, encode};
use image::{buffer, DynamicImage, GenericImageView, Rgba};
use serde::{Deserialize, Serialize};
use show_image::*;
use std::collections::HashSet as Hashset;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::{process, str, thread};
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;

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

#[derive(Serialize, Deserialize, Debug)]
struct MessageType {
    message: String,
    id: u8,
    image_fragment: Vec<u8>,
    views: i32,
}

fn open_image(image_path: &str) {
    // Load the image file
    let img = image::open(image_path).unwrap();

    // Convert the image to RGBA format
    let rgba_image = img.to_rgba8();

    // Get image dimensions
    let (width, height) = rgba_image.dimensions();

    // Convert the image to a flat vector of u8 pixel data
    let pixel_data: Vec<u8> = rgba_image.into_raw();

    // Create an ImageView with the loaded image data
    let image = ImageView::new(ImageInfo::rgba8(width, height), &pixel_data);

    // Create a window with default options and display the image
    let window = create_window("image", Default::default()).unwrap();
    window.set_image("image-001", image);

    thread::sleep(Duration::from_secs(4));

    // delete_image(image_path);
}

fn delete_image(image_path: &str) {
    fs::remove_file(image_path);
}

fn print_DOS(directory_of_service: &Hashset<String>) {
    println!("Directory of service:");
    let mut num = 1;
    for ip in directory_of_service.clone() {
        println!("{}: {}", num, ip);
        num += 1;
    }
}

#[show_image::main]
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

    // between clients
    let listening_port = 5555;
    let sending_port = 6666;
    let client_send_socket = create_socket(client_ip, sending_port);
    let client_listen_socket = create_socket(client_ip, listening_port);

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

    // directory of service is a hashset of IPv4 addresses
    let mut directory_of_service: Hashset<String> = Hashset::new();

    // all encoded images vector
    let mut all_encoded_images: Vec<Vec<u8>> = Vec::new();

    // load my image and convert it to bytes
    let mut payload = File::open("big.png").unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();

    // fragment the image into bytes
    let mut fragmented_image_bytes = Vec::new();
    for chunk in payload_bytes.chunks(1024) {
        fragmented_image_bytes.push(chunk);
    }

    for i in 1..3 {
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

                if j % 15 == 0 {
                    // sleep for 1 second
                    thread::sleep(Duration::from_millis(20));
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

                // the recieved chunk has IPs separated by newlines
                let directory = str::from_utf8(&recieved_chunk).unwrap();
                let directory = directory.split("\n");
                for ip in directory {
                    // make sure its not an empty string
                    if ip == "" || ip == client_ip {
                        continue;
                    }
                    // add to directory of service with listening port
                    let ip = format!("{}:{}", ip, listening_port);
                    directory_of_service.insert(ip);
                }
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
        let filename = format!("encoded_image_{}_client_{}.png", i, client_num);
        let mut file = File::create(filename.clone()).unwrap();
        file.write_all(&reconstructed_image_bytes);
        all_encoded_images.push(reconstructed_image_bytes);

        // decode the file
        let encoded_image = file_as_image_buffer(filename.to_string());
        let dec = Decoder::new(encoded_image);
        let out_buffer = dec.decode_alpha();
        let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
        let message = bytes_to_str(clean_buffer.as_slice());
        let decoded_image = base64::decode(message).unwrap();
        let path = format!("decoded_image_{}_client_{}.png", i, client_num);
        let mut file = File::create(path).unwrap();
        file.write_all(&decoded_image);
    }

    // print directory of service
    println!("Directory of service:");
    let mut num = 1;
    for ip in directory_of_service.clone() {
        println!("{}: {}", num, ip);
        num += 1;
    }
    // print number of images
    println!("Number of encoded images: {}", all_encoded_images.len());

    // vector of image path and number of views recieved
    let mut all_images_recieved: Vec<(String, i32)> = Vec::new();

    //////////////////////////////////////////////////////////////////

    // this is the thread responsible for sending back the total number of images to the requesting client.
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let tx_clone = mpsc::Sender::clone(&tx);

    let client_listen_copy = client_listen_socket.try_clone().unwrap();
    let client_send_copy = client_send_socket.try_clone().unwrap();
    let mut reconstructed_image_bytes: Vec<u8> = Vec::new();

    thread::spawn(move || {
        loop {
            // listen for messages from the requesting client
            let mut buffer = [0; 65535];
            let (amt, src) = client_listen_copy
                .recv_from(&mut buffer)
                .expect("Didn't receive data");
            let encoded = str::from_utf8(&buffer[..amt]).unwrap();
            let message: MessageType = serde_json::from_str(encoded).unwrap();
            let msg = message.message;
            let id = message.id;
            let image_fragment = message.image_fragment;
            let views = message.views;
            let src = src.ip();
            let src = format!("{}:{}", src, listening_port);

            if id == 1 {
                // this is the first message. send the number of images.
                // send the number of images to the requesting client
                let num_images = all_encoded_images.len().to_string();
                let message = MessageType {
                    message: num_images,
                    id: 2,
                    image_fragment: Vec::new(),
                    views: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                println!("Sending number of images to requesting client");
                client_send_copy
                    .send_to(encoded.as_bytes(), &src)
                    .expect("Failed to send data to server");
            }
            if id == 2 {
                // this is the second message. recieves the number of images
                let num_images = msg.parse::<usize>().unwrap();
                // choose a random image
                println!("NUM IMAGES: {}", num_images);
                println!("Enter the number of the image you want to send:");
                let mut image_to_send = String::new();
                std::io::stdin()
                    .read_line(&mut image_to_send)
                    .expect("Failed to read line");
                let image_to_send = image_to_send.trim().parse::<usize>().unwrap();

                //send that number to the sending client that will send us the image.
                let message = MessageType {
                    message: image_to_send.to_string(),
                    id: 3,
                    image_fragment: Vec::new(),
                    views: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                println!("Sending to src: {}", src);
                // this is sent to the requesting client
                client_send_copy
                    .send_to(encoded.as_bytes(), &src)
                    .expect("Failed to send data to server");
                println!("Sent image number to requesting client");
            }
            if id == 3 {
                // this is the third message. recieves the image number and sends the image to the requesting client.
                // recieve the image number from the requesting client
                let image_to_send = msg.parse::<usize>().unwrap();
                // send the image to the requesting client
                let encoded_image = all_encoded_images[image_to_send - 1].clone();
                println!("Sending image to requesting client");
                let mut encoded_image_chunks = Vec::new();
                for chunk in encoded_image.chunks(1024) {
                    encoded_image_chunks.push(chunk);
                }
                // for every chunk make an instance of messageType
                for j in 0..encoded_image_chunks.len() {
                    // send the struct to the client
                    let image_fragment = MessageType {
                        message: String::new(),
                        id: 4,
                        image_fragment: encoded_image_chunks[j].to_vec(),
                        views: 0,
                    };
                    let encoded = serde_json::to_string(&image_fragment).unwrap();
                    // send to the requesting client
                    client_send_copy
                        .send_to(encoded.as_bytes(), &src)
                        .expect("Failed to send data to server");
                    if j % 15 == 0 && j != 0 {
                        // sleep for 1 second
                        thread::sleep(Duration::from_millis(30));
                    }
                }
                // send end to the requesting client
                let end_message = "MINSENDEND";
                let final_message = MessageType {
                    message: end_message.to_string(),
                    id: 4,
                    image_fragment: Vec::new(),
                    views: 3,
                };
                let encoded = serde_json::to_string(&final_message).unwrap();
                client_send_copy
                    .send_to(encoded.as_bytes(), &src)
                    .expect("Failed to send data to server");
            }
            if id == 4 {
                // reconstruct the image from the chunks
                println!("Receiving image from client: {}", src);
                reconstructed_image_bytes.append(&mut image_fragment.clone());
                if msg == "MINSENDEND" {
                    println!("Reconstructed image from client: {}", src);
                    // write the image to a file
                    let filename = format!("reconstructed_image_client_{}.png", client_num);
                    let mut file = File::create(filename.clone()).unwrap();
                    file.write_all(&reconstructed_image_bytes);
                    println!("Reconstructed image from client: {}", src);
                    // add to all images recieved
                    let image_info = (filename, views);
                    all_images_recieved.push(image_info);

                    loop {
                        // ask the user if they want to view images or request another image or add views
                        println!("1. to view images.");
                        println!("2. to request another image.");
                        println!("3. to add views to an image.");
                        println!("4. to exit.");
                        let mut choice = String::new();
                        std::io::stdin()
                            .read_line(&mut choice)
                            .expect("Failed to read line");
                        let choice = choice.trim().parse::<u8>().unwrap();
                        match choice {
                            1 => {
                                // view images
                                for i in 0..all_images_recieved.len() {
                                    println!(
                                        "{}. {} with views {}",
                                        i + 1,
                                        all_images_recieved[i].0,
                                        all_images_recieved[i].1
                                    );
                                }
                                println!("Enter the number of the image you want to view:");
                                let mut image_to_view = String::new();
                                std::io::stdin()
                                    .read_line(&mut image_to_view)
                                    .expect("Failed to read line");
                                let image_to_view = image_to_view.trim().parse::<usize>().unwrap();
                                let image_to_view = image_to_view - 1;
                                let image_to_view1 = all_images_recieved[image_to_view].0.clone();
                                if (all_images_recieved[image_to_view].1) == 0 {
                                    println!("You do not have access to this image");
                                } else {
                                    // decode the image
                                    let encoded_image =
                                        file_as_image_buffer(image_to_view1.clone());
                                    let dec = Decoder::new(encoded_image);
                                    let out_buffer = dec.decode_alpha();
                                    let clean_buffer: Vec<u8> =
                                        out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
                                    let message = bytes_to_str(clean_buffer.as_slice());
                                    let decoded_image = base64::decode(message).unwrap();
                                    let path = format!(
                                        "decoded_image_{}_client_{}_finalview.png",
                                        image_to_view, client_num
                                    );
                                    let mut file = File::create(path.clone()).unwrap();
                                    file.write_all(&decoded_image);
                                    open_image(&path);
                                    delete_image(&path);
                                    // decrement the views of the image
                                    let mut views = all_images_recieved[image_to_view].1;
                                    views -= 1;
                                    all_images_recieved[image_to_view].1 = views;
                                }
                            }
                            2 => {
                                tx_clone.send("request image".to_string()).unwrap();
                                break;
                            }
                            3 => {
                                // add views to an image
                                // send the number of images to the requesting client
                                let message = MessageType {
                                    message: "change views".to_string(),
                                    id: 5,
                                    image_fragment: Vec::new(),
                                    views: 0,
                                };
                                let encoded = serde_json::to_string(&message).unwrap();
                                println!("Sending number of images to requesting client");
                                client_send_copy
                                    .send_to(encoded.as_bytes(), &src)
                                    .expect("Failed to send data to server");
                                // recieve the image number from the requesting client
                                let image_to_send = msg.parse::<usize>().unwrap();
                                // send the image to the requesting client
                                let encoded_image = all_encoded_images[image_to_send - 1].clone();
                                println!("Sending image to requesting client");
                                let mut encoded_image_chunks = Vec::new();
                                for chunk in encoded_image.chunks(1024) {
                                    encoded_image_chunks.push(chunk);
                                }
                                // for every
                            }
                            4 => {
                                // exit
                                process::exit(0);
                            }
                            _ => {
                                println!("Invalid choice");
                            }
                        }
                    }
                }
            }
        }
    });

    let mut message_count = 0;

    loop {
        // MAIN THREAD
        if message_count != 0 {
            // recieve from channel
            let recieved = rx.recv().unwrap();
            if recieved != "request image" {
                continue;
            }
        }
        println!("Enter the number of the client you want to send to:");
        let mut client_to_send_to = String::new();
        std::io::stdin()
            .read_line(&mut client_to_send_to)
            .expect("Failed to read line");
        let client_to_send_to = client_to_send_to.trim().parse::<u16>().unwrap();

        // get the ip of the client to send to from the directory of service
        let mut client_to_send_to_ip = String::new();
        let mut num = 1;
        for ip in directory_of_service.clone() {
            if num == client_to_send_to {
                client_to_send_to_ip = ip;
                break;
            }
            num += 1;
        }
        // send to that client "HELLO"
        let hello_message = "HELLO";
        let id = 1;
        let message = MessageType {
            message: hello_message.to_string(),
            id: id,
            image_fragment: Vec::new(),
            views: 0,
        };
        let encoded = serde_json::to_string(&message).unwrap();
        client_send_socket
            .send_to(encoded.as_bytes(), &client_to_send_to_ip)
            .expect("Failed to send data to server");
        message_count += 1;
    }
    // never stop
    loop {}

    // if client_num == 1 {
    //     thread::sleep(Duration::from_millis(3000));
    //     // sleep for 2 seconds to make sure all clients are listening
    //     println!("Sending image to client 2");
    //     // send message to clients in directory of service
    //     for ip in directory_of_service {
    //         // choose who to send to. For now, send to client 2
    //         if ip != "127.0.0.5:5555" {
    //             continue;
    //         }
    //         // send the first encoded image to client
    //         let encoded_image = all_encoded_images[0].clone();
    //         let mut encoded_image_chunks = Vec::new();
    //         for chunk in encoded_image.chunks(1024) {
    //             encoded_image_chunks.push(chunk);
    //         }
    //         for j in 0..encoded_image_chunks.len() {
    //             // send the struct to the server
    //             let image_fragment = ImageViews {
    //                 fragment: encoded_image_chunks[j].to_vec(),
    //                 access_rights: 2,
    //             };

    //             let encoded = serde_json::to_string(&image_fragment).unwrap();

    //             // send to client 2
    //             client_send_socket
    //                 .send_to(encoded.as_bytes(), &ip)
    //                 .expect("Failed to send data to server");

    //             if j % 20 == 0 && j != 0 {
    //                 // sleep for 1 second
    //                 thread::sleep(Duration::from_millis(20));
    //             }
    //         }
    //         // send end to client 2
    //         let end_message = "MINSENDEND";
    //         let final_message = ImageViews {
    //             fragment: end_message.as_bytes().to_vec(),
    //             access_rights: 2,
    //         };
    //         let encoded = serde_json::to_string(&final_message).unwrap();
    //         client_send_socket
    //             .send_to(encoded.as_bytes(), &ip)
    //             .expect("Failed to send data to server");
    //         // remove the port number from the ip
    //         let ip = ip.split(":").collect::<Vec<&str>>()[0];
    //         // add to all images sent
    //         let image_info = ImageClientInfo {
    //             access_rights: 2,
    //             sender_ip: client_ip.to_string(),
    //             reciever_ip: ip.to_string(),
    //             // image_data: all_encoded_images[0].clone(),
    //         };
    //         all_images_sent.push(image_info);
    //         println!("Sent image to client 2");

    //         // print the all_images_sent vector
    //         println!("All images sent:");
    //         println!("{:?}", all_images_sent);
    //     }
    // } else if client_num == 2 {
    //     // listen for messages from clients
    //     let mut buffer = [0; 65535];
    //     let mut recieved_image: Vec<u8> = Vec::new();
    //     let mut views = 0;
    //     let mut src_ip;
    //     loop {
    //         let (amt, src) = client_listen_socket
    //             .recv_from(&mut buffer)
    //             .expect("Didn't receive data");
    //         let msg = str::from_utf8(&buffer[..amt]).unwrap();

    //         let image_fragment: ImageViews = serde_json::from_str(msg).unwrap();
    //         let recieved_chunk = image_fragment.fragment;
    //         let access_rights = image_fragment.access_rights;
    //         let src = src.ip();

    //         if recieved_chunk == b"MINSENDEND" {
    //             views = access_rights;
    //             src_ip = src;
    //             break;
    //         }
    //         recieved_image.append(&mut recieved_chunk.to_vec());
    //     }
    //     println!("Received image from client 1");
    //     // add to all images recieved
    //     let image_info = ImageClientInfo {
    //         access_rights: views as i32,
    //         sender_ip: src_ip.to_string(),
    //         reciever_ip: client_ip.to_string(),
    //         // image_data: recieved_image.clone(),
    //     };
    //     all_images_recieved.push(image_info);
    //     let mut File = File::create("recieved_image.png").unwrap();
    //     File.write_all(&recieved_image);

    //     println!("All images recieved:");
    //     println!("{:?}", all_images_recieved);

    //     ////////////////////////////////////////////////////

    //     // decode the image view times
    //     for i in 0..views {
    //         let encoded_image = file_as_image_buffer("recieved_image.png".to_string());
    //         let dec = Decoder::new(encoded_image);
    //         let out_buffer = dec.decode_alpha();
    //         let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
    //         let message = bytes_to_str(clean_buffer.as_slice());
    //         let decoded_image = base64::decode(message).unwrap();
    //         let path = format!("FINALVIEWS_image_{}_client_{}.png", i, client_num);
    //         let mut file = File::create(path.clone()).unwrap();
    //         file.write_all(&decoded_image);
    //         // open_image(&path);
    //     }
    // }

    // if client_num == 1 {
    //     // loop over the sent images vector and update the access rights of the first image
    //     for i in 0..all_images_sent.len() {
    //         if all_images_sent[i].reciever_ip == client_2 {
    //             all_images_sent[i].access_rights = 1;
    //             // send to client 2 this updated access rights number
    //             // add the port number to the ip
    //             let client_2 = format!("{}:{}", client_2, listening_port);
    //             client_send_socket
    //                 .send_to(
    //                     all_images_sent[i].access_rights.to_string().as_bytes(),
    //                     &client_2,
    //                 )
    //                 .expect("Failed to send data to server");
    //         }
    //     }
    // } else if client_num == 2 {
    //     // listen for messages from clients
    //     let mut buffer = [0; 65535];
    //     let mut recieved_access_rights: Vec<u8> = Vec::new();
    //     let mut src_ip;
    //     loop {
    //         let (amt, src) = client_listen_socket
    //             .recv_from(&mut buffer)
    //             .expect("Didn't receive data");
    //         let msg = str::from_utf8(&buffer[..amt]).unwrap();
    //         let access_rights = msg.parse::<u8>().unwrap();
    //         src_ip = src.ip();
    //         recieved_access_rights.push(access_rights);
    //         if recieved_access_rights.len() == 1 {
    //             break;
    //         }
    //     }
    //     // loop over the recieved images vector and update the access rights of the first image
    //     for i in 0..all_images_recieved.len() {
    //         if all_images_recieved[i].sender_ip == client_1 {
    //             all_images_recieved[i].access_rights = recieved_access_rights[0] as i32;
    //         }
    //     }
    //     println!("All images recieved:");
    //     println!("{:?}", all_images_recieved);
    // }
}
