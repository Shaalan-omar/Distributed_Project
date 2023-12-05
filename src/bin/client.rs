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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{mem, process, str, thread};
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
    name: String,
    is_sample: bool,
    sample_num: u8,
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

    thread::sleep(Duration::from_secs(2));

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

    let client_1 = "127.0.0.4"; // MINS HP
    let client_2 = "127.0.0.5"; // SHAALAN HP
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

    let server_1_socket = "127.0.0.1:3333"; // SHAALAN MACBOOK
    let server_2_socket = "127.0.0.2:3333"; // ZIZO YOGA
    let server_3_socket = "127.0.0.3:3333"; // ZIZO THINKPAD

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

    // compressed image vector
    let mut all_compressed_images: Vec<Vec<u8>> = Vec::new();
    // fill it up with the big_compressed.png and pic2_compressed.png
    let mut payload = File::open("big_compressed.png").unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();
    all_compressed_images.push(payload_bytes);
    let mut payload = File::open("pic2_compressed.png").unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();
    all_compressed_images.push(payload_bytes);

    // for i in 1..4 {
    // if i == 3 {
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
    // } else if i == 1 {
    //     let mut fragmented_image_bytes = Vec::new();
    //     // load my image and convert it to bytes
    //     let mut payload = File::open("big.png").unwrap();
    //     let mut payload_bytes = Vec::new();
    //     payload.read_to_end(&mut payload_bytes).unwrap();

    //     // fragment the image into bytes
    //     for chunk in payload_bytes.chunks(1024) {
    //         fragmented_image_bytes.push(chunk);
    //     }

    //     // else send the image to all servers.
    //     for j in 0..fragmented_image_bytes.len() {
    //         // send the struct to the server
    //         let image_fragment = ImageFragment {
    //             fragment: fragmented_image_bytes[j].to_vec(),
    //             request_type: request_type_image,
    //         };

    //         let encoded = serde_json::to_string(&image_fragment).unwrap();

    //         // send to server1
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_1_socket)
    //             .expect("Failed to send data to server");
    //         // send to server2
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_2_socket)
    //             .expect("Failed to send data to server");
    //         // send to server3
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_3_socket)
    //             .expect("Failed to send data to server");

    //         if j % 15 == 0 {
    //             // sleep for 1 second
    //             thread::sleep(Duration::from_millis(20));
    //         }
    //     }
    //     println!("Sent picture number {} to all servers", i);

    //     // send end to all servers
    //     let end_message = "MINSENDEND";
    //     let final_message = ImageFragment {
    //         fragment: end_message.as_bytes().to_vec(),
    //         request_type: request_type_image,
    //     };
    //     let encoded = serde_json::to_string(&final_message).unwrap();

    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_1_socket)
    //         .expect("Failed to send data to server");
    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_2_socket)
    //         .expect("Failed to send data to server");
    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_3_socket)
    //         .expect("Failed to send data to server");
    //     println!("Sent end to all servers");
    // } else if i == 2 {
    //     let mut fragmented_image_bytes = Vec::new();
    //     // load my image and convert it to bytes
    //     let mut payload = File::open("pic2.png").unwrap();
    //     let mut payload_bytes = Vec::new();
    //     payload.read_to_end(&mut payload_bytes).unwrap();

    //     // fragment the image into bytes
    //     for chunk in payload_bytes.chunks(1024) {
    //         fragmented_image_bytes.push(chunk);
    //     }

    //     // else send the image to all servers.
    //     for j in 0..fragmented_image_bytes.len() {
    //         // send the struct to the server
    //         let image_fragment = ImageFragment {
    //             fragment: fragmented_image_bytes[j].to_vec(),
    //             request_type: request_type_image,
    //         };

    //         let encoded = serde_json::to_string(&image_fragment).unwrap();

    //         // send to server1
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_1_socket)
    //             .expect("Failed to send data to server");
    //         // send to server2
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_2_socket)
    //             .expect("Failed to send data to server");
    //         // send to server3
    //         sending_socket
    //             .send_to(encoded.as_bytes(), &server_3_socket)
    //             .expect("Failed to send data to server");

    //         if j % 15 == 0 {
    //             // sleep for 1 second
    //             thread::sleep(Duration::from_millis(20));
    //         }
    //     }
    //     println!("Sent picture number {} to all servers", i);

    //     // send end to all servers
    //     let end_message = "MINSENDEND";
    //     let final_message = ImageFragment {
    //         fragment: end_message.as_bytes().to_vec(),
    //         request_type: request_type_image,
    //     };
    //     let encoded = serde_json::to_string(&final_message).unwrap();

    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_1_socket)
    //         .expect("Failed to send data to server");
    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_2_socket)
    //         .expect("Failed to send data to server");
    //     sending_socket
    //         .send_to(encoded.as_bytes(), &server_3_socket)
    //         .expect("Failed to send data to server");
    //     println!("Sent end to all servers");
    // }

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
    // if !isimage {
    //     isimage = true;
    //     continue;
    // }

    // println!("Received encyrption from server: {}", src_server);
    // let mut reconstructed_image_bytes = Vec::new();
    // for k in 0..image_from_server.len() {
    //     reconstructed_image_bytes.push(image_from_server[k]);
    // }
    // let filename = format!("encoded_image_{}_client_{}.png", i, client_num);
    // let mut file = File::create(filename.clone()).unwrap();
    // file.write_all(&reconstructed_image_bytes);
    // all_encoded_images.push(reconstructed_image_bytes);

    // // decode the file
    // let encoded_image = file_as_image_buffer(filename.to_string());
    // let dec = Decoder::new(encoded_image);
    // let out_buffer = dec.decode_alpha();
    // let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
    // let message = bytes_to_str(clean_buffer.as_slice());
    // let decoded_image = base64::decode(message).unwrap();
    // let path = format!("decoded_image_{}_client_{}.png", i, client_num);
    // let mut file = File::create(path).unwrap();
    // file.write_all(&decoded_image);
    // sleep for 2 seconds
    thread::sleep(Duration::from_secs(2));
    // }

    // print directory of service
    println!("Directory of service:");
    let mut num = 1;
    for ip in directory_of_service.clone() {
        println!("{}: {}", num, ip);
        num += 1;
    }
    // print number of images

    let filename = format!("C:/Users/demim/OneDrive/Desktop/Uni/Fall 2023/Fundamentals of Distributed Systems/proj/Distributed_Project/encoded_image_1_client_{}.png", client_num);
    let mut file = File::open(filename).unwrap();
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes).unwrap();
    all_encoded_images.push(file_bytes);
    let filename = format!("C:/Users/demim/OneDrive/Desktop/Uni/Fall 2023/Fundamentals of Distributed Systems/proj/Distributed_Project/encoded_image_2_client_{}.png", client_num);
    let mut file = File::open(filename).unwrap();
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes).unwrap();
    all_encoded_images.push(file_bytes);
    println!("Number of encoded images: {}", all_encoded_images.len());

    // vector of image path and number of views recieved
    // (image path, views, image number, who sent it)
    // let mut all_images_recieved: Vec<(String, i32, i32, String)> = Vec::new();
    let mut all_images_recieved: Arc<Mutex<Vec<(String, i32, i32, String)>>> =
        Arc::new(Mutex::new(Vec::new()));
    let all_images_recieved_clone = Arc::clone(&all_images_recieved);

    // vector of pairs that has image id and destination ip
    let mut all_images_sent: Arc<Mutex<Vec<(i32, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let all_images_sent_clone = Arc::clone(&all_images_sent);
    let mut offline_clients: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let offline_clients_clone = Arc::clone(&offline_clients);

    //////////////////////////////////////////////////////////////////

    // this is the thread responsible for sending back the total number of images to the requesting client.
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let tx_clone = mpsc::Sender::clone(&tx);

    let client_listen_copy = client_listen_socket.try_clone().unwrap();
    let client_send_copy = client_send_socket.try_clone().unwrap();
    let server_send_copy = sending_socket.try_clone().unwrap();
    let server_listen_copy = recieving_socket.try_clone().unwrap();
    let mut reconstructed_image_bytes: Vec<u8> = Vec::new();

    let mut img_counter: u16 = 1;
    let mut go_to_id_4 = false;
    let mut compressed_images_recieved: Vec<Vec<u8>> = Vec::new();

    thread::spawn(move || {
        loop {
            // listen for messages from the requesting client
            let mut buffer = [0; 65535];
            let mut src = String::new();
            let mut encoded: &str;
            let mut message: MessageType;
            let mut msg: String = String::new();
            let mut id: u8 = 128;
            let mut image_fragment = Vec::new();
            let mut views = 20000;
            let mut name = String::new();
            let mut is_sample;
            let mut sample_num;

            if go_to_id_4 == false {
                let (amt, src1) = client_listen_copy
                    .recv_from(&mut buffer)
                    .expect("Didn't receive data");
                encoded = str::from_utf8(&buffer[..amt]).unwrap();
                message = serde_json::from_str(encoded).unwrap();
                msg = message.message;
                id = message.id;
                image_fragment = message.image_fragment;
                views = message.views;
                name = message.name;
                is_sample = message.is_sample;
                sample_num = message.sample_num;
                src = src1.ip().to_string();
                src = format!("{}:{}", src, listening_port);
            }

            if id == 1 {
                // this is the first message. send the number of images.
                // send the compressed images to the requesting client
                let num_images = all_encoded_images.len().to_string();
                println!("Sending compressed images to client");
                for i in 0..all_compressed_images.len() {
                    // send the struct to the client
                    // if its the last image, send the end message
                    let mut image_fragment: MessageType;
                    if i == all_compressed_images.len() - 1 {
                        image_fragment = MessageType {
                            message: String::new(),
                            id: 2,
                            image_fragment: all_compressed_images[i].clone(),
                            views: 1000,
                            name: "".to_string(),
                            is_sample: true,
                            sample_num: i as u8,
                        };
                    } else {
                        image_fragment = MessageType {
                            message: String::new(),
                            id: 2,
                            image_fragment: all_compressed_images[i].clone(),
                            views: 0,
                            name: "".to_string(),
                            is_sample: true,
                            sample_num: i as u8,
                        };
                    }
                    let encoded = serde_json::to_string(&image_fragment).unwrap();
                    // send to the requesting client
                    client_send_copy
                        .send_to(encoded.as_bytes(), &src)
                        .expect("Failed to send data to server");
                }
            }
            if id == 2 {
                // this is the second message. recieves the compressed images.
                // saves them into a vector.
                // if the last image is sent, whe views = 1000.
                // open both images and ask the user which one they want to request.

                let mut compressed_image = image_fragment.clone();
                if views != 1000 {
                    compressed_images_recieved.push(compressed_image.clone());
                    continue;
                }
                compressed_images_recieved.push(compressed_image.clone());

                // open both compressed images and ask the user which one they want to request.
                let mut num = 1;
                for image in compressed_images_recieved.clone() {
                    let path = format!(
                        "compressed_image_{}_client_{}_{}.png",
                        num, client_num, name
                    );
                    let mut file = File::create(path.clone()).unwrap();
                    file.write_all(&image);
                    open_image(&path);
                    delete_image(&path);
                    num += 1;
                }
                // clear the compressed images recieved vector
                compressed_images_recieved.clear();
                // ask the user which image they want to request
                println!("Enter the number of the image you want to request:");
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
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
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
                        name: "".to_string(),
                        is_sample: false,
                        sample_num: 0,
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
                    name: img_counter.to_string(),
                    is_sample: false,
                    sample_num: 0,
                };
                let encoded = serde_json::to_string(&final_message).unwrap();
                client_send_copy
                    .send_to(encoded.as_bytes(), &src)
                    .expect("Failed to send data to server");
                // add to all images sent
                let image_info = (img_counter as i32, src.clone());
                let mut all_images_sent = all_images_sent_clone.lock().unwrap();
                all_images_sent.push(image_info);
                img_counter += 1;
            }
            if id == 4 || go_to_id_4 == true {
                // reconstruct the image from the chunks
                println!("Receiving image from client: {}", src);
                reconstructed_image_bytes.append(&mut image_fragment.clone());
                if msg == "MINSENDEND" || go_to_id_4 == true {
                    if (go_to_id_4 == false) {
                        println!("Reconstructed image from client: {}", src);
                        // write the image to a file
                        let filename =
                            format!("reconstructed_image_client_{}_{}.png", client_num, name);
                        let mut file = File::create(filename.clone()).unwrap();
                        file.write_all(&reconstructed_image_bytes);
                        println!("Reconstructed image from client: {}", src);
                        // clear the reconstructed image bytes vector
                        reconstructed_image_bytes.clear();
                        // add to all images recieved
                        let image_info =
                            (filename, views, name.parse::<i32>().unwrap(), src.clone());
                        let mut all_images_recieved = all_images_recieved_clone.lock().unwrap();
                        all_images_recieved.push(image_info);
                    }
                    go_to_id_4 = false;
                    println!("HEREHRE");
                    loop {
                        // ask the user if they want to view images or request another image or add views
                        println!("1. to view images.");
                        println!("2. to request another image.");
                        println!("3. to renew views of an image.");
                        println!("4. to exit.");
                        let mut choice = String::new();
                        std::io::stdin()
                            .read_line(&mut choice)
                            .expect("Failed to read line");
                        let choice = choice.trim().parse::<u8>().unwrap();
                        let mut all_images_recieved = all_images_recieved_clone.lock().unwrap();

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
                                // ask the user which image they want to add views to
                                for i in 0..all_images_recieved.len() {
                                    println!(
                                        "{}. {} with views {}",
                                        i + 1,
                                        all_images_recieved[i].0,
                                        all_images_recieved[i].1
                                    );
                                }
                                println!("Enter the number of the image you want to add views to:");
                                let mut image_to_add_views = String::new();
                                std::io::stdin()
                                    .read_line(&mut image_to_add_views)
                                    .expect("Failed to read line");

                                let image_to_add_views1 =
                                    image_to_add_views.trim().parse::<usize>().unwrap();
                                let image_to_add_views1 = image_to_add_views1 - 1;
                                let image_to_add_views = all_images_recieved[image_to_add_views1].2;
                                let client_to_send_to_ip = all_images_recieved[image_to_add_views1]
                                    .3
                                    .parse::<SocketAddr>()
                                    .unwrap();

                                let message = MessageType {
                                    message: "".to_string(),
                                    id: 6,
                                    image_fragment: Vec::new(),
                                    views: 0,
                                    name: image_to_add_views.to_string(),
                                    is_sample: false,
                                    sample_num: 0,
                                };
                                let encoded = serde_json::to_string(&message).unwrap();
                                // this is sent to the address of the client that sent the image
                                client_send_copy
                                    .send_to(encoded.as_bytes(), client_to_send_to_ip)
                                    .expect("Failed to send data to server");

                                break;
                            }
                            4 => {
                                // exit
                                break;
                            }
                            _ => {
                                println!("Invalid choice");
                            }
                        }
                    }
                }
            }
            if id == 5 {
                let image_to_change_views = name.parse::<i32>().unwrap();
                let new_views = views;
                let mut all_images_recieved = all_images_recieved_clone.lock().unwrap();
                for i in 0..all_images_recieved.len() {
                    if all_images_recieved[i].2 == image_to_change_views {
                        mem::replace(&mut all_images_recieved[i].1, new_views);
                        println!("Changed views of image: {}", image_to_change_views);
                        println!("New views: {}", all_images_recieved[i].1);
                        // go to id=4
                        println!("changed views");
                        go_to_id_4 = true;
                    }
                }
            }
            if id == 6 {
                println!("Client {} wants to add views to image {}", src, name);
                println!("1. to approve.");
                println!("2. to decline.");

                let mut choice = String::new();
                std::io::stdin()
                    .read_line(&mut choice)
                    .expect("Failed to read line");
                let choice = choice.trim().parse::<u8>().unwrap();

                let message: MessageType;
                if choice == 1 {
                    // approve
                    // send a yes message to the sending client
                    message = MessageType {
                        message: "yes".to_string(),
                        id: 7,
                        image_fragment: Vec::new(),
                        views: 3,
                        name: name.clone(),
                        is_sample: false,
                        sample_num: 0,
                    };
                } else {
                    // decline
                    message = MessageType {
                        message: "no".to_string(),
                        id: 7,
                        image_fragment: Vec::new(),
                        views: 0,
                        name: name.clone(),
                        is_sample: false,
                        sample_num: 0,
                    };
                }

                tx_clone.send("change views".to_string()).unwrap();

                println!("Sending message to client: {}", src);
                println!("Message: {}", message.message);

                let encoded = serde_json::to_string(&message).unwrap();
                // this is sent to the address of the client that sent the image
                client_send_copy
                    .send_to(encoded.as_bytes(), &src)
                    .expect("Failed to send data to server");
            }
            if id == 7 {
                if msg == "yes" {
                    // add views to the image
                    let image_to_change_views = name.parse::<i32>().unwrap();
                    let new_views = views;
                    let mut all_images_recieved = all_images_recieved_clone.lock().unwrap();
                    for i in 0..all_images_recieved.len() {
                        if all_images_recieved[i].2 == image_to_change_views {
                            mem::replace(&mut all_images_recieved[i].1, new_views);
                            println!("Changed views of image: {}", image_to_change_views);
                            println!("New views: {}", all_images_recieved[i].1);
                            // go to id=4
                            println!("changed views");
                        }
                    }
                } else {
                    println!("Client {} declined to add views to image {}", src, name);
                }
                go_to_id_4 = true;
            }
            if id == 8 {
                // send to server that the src client is offline
                let offline_message = "OFFLINE";
                // add the offline client to the vector of offline clients
                let mut offline_clients = offline_clients_clone.lock().unwrap();
                offline_clients.push(src.clone());

                let mut src_to_send = src.clone();
                // remove the port number
                src_to_send.truncate(src_to_send.len() - 5);
                // add the port number 9999
                src_to_send = format!("{}:{}", src_to_send, 9999);

                // send to server offline message and with it the IP and PORT of client that went offline

                let message = MessageType {
                    message: src_to_send.clone(),
                    id: 111,
                    image_fragment: Vec::new(),
                    views: 0,
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
                };

                let encoded = serde_json::to_string(&message).unwrap();
                // this is sent to the address of the client that sent the image
                server_send_copy
                    .send_to(encoded.as_bytes(), &server_1_socket)
                    .expect("Failed to send data to server");
                server_send_copy
                    .send_to(encoded.as_bytes(), &server_2_socket)
                    .expect("Failed to send data to server");
                server_send_copy
                    .send_to(encoded.as_bytes(), &server_3_socket)
                    .expect("Failed to send data to server");
            }
            if id == 9 {
                // remove from the offline clients vector
                let mut offline_clients = offline_clients_clone.lock().unwrap();
                for i in 0..offline_clients.len() {
                    if offline_clients[i] == src {
                        offline_clients.remove(i);
                        println!("Client {} is back online", src);
                        break;
                    }
                }
            }
        }
    });

    let mut message_count = 0;
    let mut hold_input = false;
    let mut go_online = false;
    let mut skipthis = false;

    loop {
        // MAIN THREAD
        if go_online == false {
            if hold_input == true {
                // recieve from channel
                let recieved = rx.recv().unwrap();
                hold_input = false;
            } else if message_count != 0 {
                // recieve from channel
                if skipthis == false {
                    let recieved = rx.recv().unwrap();
                    if recieved != "request image" {
                        continue;
                    }
                }
            }
        }
        skipthis = false;
        if go_online == true {
            println!("5. Simulate going online.");
        } else {
            // do u want to send to client or change views of a sent image
            println!("1. Request from client.");
            println!("2. Change views of a sent image.");
            println!("3. Accept remote changing of views.");
            println!("4. Simulate going offline.");
        }

        let mut choice = String::new();
        std::io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice = choice.trim().parse::<u8>().unwrap();

        match choice {
            1 => {
                println!("Enter the number of the client you want to send to:");
                let mut client_to_send_to = String::new();
                std::io::stdin()
                    .read_line(&mut client_to_send_to)
                    .expect("Failed to read line");
                let client_to_send_to = client_to_send_to.trim().parse::<u16>().unwrap();
                // if the index is greater than the number of clients, loop again

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
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
                };

                let encoded = serde_json::to_string(&message).unwrap();
                client_send_socket
                    .send_to(encoded.as_bytes(), &client_to_send_to_ip)
                    .expect("Failed to send data to server");
                message_count += 1;
            }
            2 => {
                let all_images_sent = all_images_sent.lock().unwrap();
                if all_images_sent.len() == 0 {
                    println!("You have not sent any images");
                    continue;
                }
                for i in 0..all_images_sent.len() {
                    println!(
                        "{}. {} sent to {}",
                        i + 1,
                        all_images_sent[i].0,
                        all_images_sent[i].1
                    );
                }
                println!("Enter the number of the image you want to change views of:");
                let mut image_to_change_views = String::new();
                std::io::stdin()
                    .read_line(&mut image_to_change_views)
                    .expect("Failed to read line");
                let input_choice = image_to_change_views.trim().parse::<usize>().unwrap();
                let image_to_change_views = input_choice - 1;
                let image_to_change_views = all_images_sent[image_to_change_views].0.clone();
                println!("Enter the new number of views:");
                let mut new_views = String::new();
                std::io::stdin()
                    .read_line(&mut new_views)
                    .expect("Failed to read line");
                let new_views = new_views.trim().parse::<i32>().unwrap();
                // send the number of images to the requesting client
                // get the src of the client from the all_images_sent vector
                let x = all_images_sent[input_choice - 1].1.clone();
                // remove the port number and 9999
                let mut x_clone = x.clone();
                x_clone.truncate(x_clone.len() - 5);
                // add the port number 9999
                x_clone = format!("{}:{}", x_clone, 9999);

                let message = MessageType {
                    message: x_clone.clone(),
                    id: 5,
                    image_fragment: Vec::new(),
                    views: new_views,                        // new views
                    name: image_to_change_views.to_string(), //message identification
                    is_sample: false,
                    sample_num: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                // this is sent to the address of the client that sent the image

                // if this IP is not in the offline clients vector, send the message
                let mut offline_clients = offline_clients.lock().unwrap();
                for ip in offline_clients.clone() {
                    if ip == all_images_sent[input_choice - 1].1 {
                        // send to server with id=222
                        sending_socket
                            .send_to(encoded.as_bytes(), &server_1_socket)
                            .expect("Failed to send data to server");
                        sending_socket
                            .send_to(encoded.as_bytes(), &server_2_socket)
                            .expect("Failed to send data to server");
                        sending_socket
                            .send_to(encoded.as_bytes(), &server_3_socket)
                            .expect("Failed to send data to server");
                    } else {
                        client_send_socket
                            .send_to(encoded.as_bytes(), &all_images_sent[input_choice - 1].1)
                            .expect("Failed to send data to server");
                    }
                }
                message_count += 1;
            }
            3 => {
                hold_input = true;
            }
            4 => {
                // send to all clients in DOS offline message
                let offline_message = "OFFLINE";
                let id = 8;
                let message = MessageType {
                    message: offline_message.to_string(),
                    id: id,
                    image_fragment: Vec::new(),
                    views: 0,
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                for ip in directory_of_service.clone() {
                    client_send_socket
                        .send_to(encoded.as_bytes(), &ip)
                        .expect("Failed to send data to server");
                }

                println!("OFFLINE");
                go_online = true;
            }
            5 => {
                // send to server that the src client is online
                // send to clients message with id = 9 to remove the src client from their offline clients vector
                let online_message = "ONLINE";
                let id = 9;
                let message = MessageType {
                    message: online_message.to_string(),
                    id: id,
                    image_fragment: Vec::new(),
                    views: 0,
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                for ip in directory_of_service.clone() {
                    client_send_socket
                        .send_to(encoded.as_bytes(), &ip)
                        .expect("Failed to send data to server");
                }

                // send to server
                let message_src = client_ip.clone();
                // add the port number 9999
                let message_src = format!("{}:{}", message_src, 9999);
                let message = MessageType {
                    message: message_src.clone(),
                    id: 222,
                    image_fragment: Vec::new(),
                    views: 0,
                    name: "".to_string(),
                    is_sample: false,
                    sample_num: 0,
                };
                let encoded = serde_json::to_string(&message).unwrap();
                sending_socket
                    .send_to(encoded.as_bytes(), &server_1_socket)
                    .expect("Failed to send data to server");
                sending_socket
                    .send_to(encoded.as_bytes(), &server_2_socket)
                    .expect("Failed to send data to server");
                sending_socket
                    .send_to(encoded.as_bytes(), &server_3_socket)
                    .expect("Failed to send data to server");
                println!("SENT");
                go_online = false;
                loop {
                    // wait for reply from server
                    println!("LOOP");
                    let mut buffer = [0; 65535];
                    let (amt, src) = recieving_socket
                        .recv_from(&mut buffer)
                        .expect("Didn't receive data");
                    let msg = str::from_utf8(&buffer[..amt]).unwrap();
                    let message: MessageType = serde_json::from_str(msg).unwrap();
                    let recieved_message = message.message;
                    let recieved_id = message.id;
                    let name = message.name;
                    let new_views = message.views;
                    // clean the recieved id and parse it into i32
                    let recieved_id = recieved_id.to_string();
                    let recieved_id = recieved_id.parse::<i32>().unwrap();

                    println!("RECIEVED");
                    println!("RECIEVED MESSAGE: {}", recieved_message);
                    println!("RECIEVED ID: {}", recieved_id);
                    let mut all_images_recieved = all_images_recieved.lock().unwrap();
                    let mut yessir: bool = false;
                    if recieved_id == 10 {
                        // change the views of the image in the all images recieved vector using the name
                        let name = name.parse::<i32>().unwrap();
                        for i in 0..all_images_recieved.len() {
                            // check if the name is the same
                            if all_images_recieved[i].2 == name {
                                // change the views
                                all_images_recieved[i].1 = new_views;
                                println!("Changed views of image: {}", name);
                                println!("New views: {}", all_images_recieved[i].1);
                                // go to id=4
                                println!("changed views");
                                yessir = true;
                                break;
                            }
                        }
                    }
                    if yessir == true {
                        message_count += 1;
                        skipthis = true;
                        break;
                    }
                }
            }
            _ => {
                println!("Invalid choice");
            }
        }
    }
}
