use base64::{decode, encode};
use image::GenericImageView;
use image::{DynamicImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{path, process, str, thread};
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
use sysinfo::{System, SystemExt};

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    server: u16,
    mem_usage: f32,
}

fn create_socket(server_ip: &str, port: u16) -> UdpSocket {
    let server_address = format!("{}:{}", server_ip, port);
    let socket_addr: SocketAddr = server_address
        .parse()
        .expect("Failed to parse socket address");
    UdpSocket::bind(socket_addr).expect("Failed to bind socket")
}

fn election_logic(
    server_num: u16,
    mem_usage: f32,
    initiator: u16,
    servers: &[&str],
    ports: &[u16],
    socket1: &UdpSocket,
    socket2: &UdpSocket,
) -> u16 {
    let mut winner: u16 = 0;

    //make the struct with server info; server num and mem usage
    let mut server_info = ServerInfo {
        server: server_num,
        mem_usage: mem_usage,
    };

    let server_info_str = serde_json::to_string(&server_info).unwrap();

    let mut buffer = [0; 1024];
    if server_num == initiator {
        // send to the next server in the ring the memory usage and server number of initiator.
        let temp = format!(
            "{}:{}",
            servers[((initiator - 1 + 1) % 3) as usize],
            ports[0]
        );
        socket2
            .send_to(server_info_str.as_bytes(), &temp)
            .expect("Failed to send data");
    } else if server_num == ((initiator - 1 + 1) % 3) + 1 {
        // recieve the memory usage and server number of initiator.
        let (amt, src) = socket1.recv_from(&mut buffer).expect("Didn't receive data");
        let msg = str::from_utf8(&buffer[..amt]).unwrap();
        let initiator_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // println!("Received: {:?} from {}", initiator_info, src); //

        //compare the memory usage of initiator and self and set the smaller one to smaller_server
        let mut smaller_server: ServerInfo;
        if initiator_info.mem_usage <= server_info.mem_usage {
            smaller_server = initiator_info;
        } else {
            smaller_server = server_info;
        }

        // send to (i+1) % 3, the memory usage and server number of the lowest to the next in the ring.
        let smaller_server_str = serde_json::to_string(&smaller_server).unwrap();
        let temp = format!(
            "{}:{}",
            servers[((initiator - 1 + 2) % 3) as usize],
            ports[0]
        );
        socket2
            .send_to(smaller_server_str.as_bytes(), &temp)
            .expect("Failed to send data");
    } else if server_num == ((initiator - 1 + 2) % 3) + 1 {
        // recieve the memory usage and server number of the lowest from the previous in the ring.
        let (amt, src) = socket1.recv_from(&mut buffer).expect("Didn't receive data");
        let msg = str::from_utf8(&buffer[..amt]).unwrap();
        let second_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // println!("Received: {:?} from {}", second_info, src); //

        //compare the memory usage of second and self and set the smaller one to smallest_server
        let mut smallest_server: ServerInfo;
        if second_info.mem_usage <= server_info.mem_usage {
            smallest_server = second_info;
            winner = smallest_server.server;
        } else {
            smallest_server = server_info;
            winner = smallest_server.server;
        }

        // send to (i+2) % 3 and i % 3, the memory usage and server number of the lowest to all in the ring.
        let smallest_server_str = serde_json::to_string(&smallest_server).unwrap();
        let temp1 = format!("{}:{}", servers[((initiator - 1) % 3) as usize], ports[0]);
        let temp2 = format!(
            "{}:{}",
            servers[((initiator - 1 + 1) % 3) as usize],
            ports[0]
        );
        socket2
            .send_to(smallest_server_str.as_bytes(), &temp1)
            .expect("Failed to send data");
        socket2
            .send_to(smallest_server_str.as_bytes(), &temp2)
            .expect("Failed to send data");
    }

    if server_num == ((initiator - 1) % 3) + 1 || server_num == ((initiator - 1 + 1) % 3) + 1 {
        // recieve the memory usage and server number of the lowest from the last in the ring.
        let (amt, src) = socket1.recv_from(&mut buffer).expect("Didn't receive data");
        let msg = str::from_utf8(&buffer[..amt]).unwrap();
        let winner_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // println!("Received: {:?} from {}", winner_info, src); //
        winner = winner_info.server;
    }

    return winner;
}

fn main() {
    let server_num: u16 = std::env::args()
        .nth(1)
        .expect("didn't specify which port")
        .parse()
        .unwrap();

    let server_1 = "127.0.0.1";
    let server_2 = "127.0.0.2";
    let server_3 = "127.0.0.3";

    let servers = vec![server_1, server_2, server_3];

    let server_ip = match server_num {
        1 => server_1,
        2 => server_2,
        3 => server_3,
        _ => panic!("Invalid server number"),
    };

    // get the memory usage per server
    let mut system = System::new_all();
    system.refresh_all();

    let mut mem_usage: f32;

    if server_num == 1 {
        // let total_mem1 = system.total_memory();
        // let mem1 = system.used_memory();
        // mem_usage = mem1 as f32 / total_mem1 as f32;
        mem_usage = 1.0;
    } else if server_num == 2 {
        // let total_mem2 = system.total_memory();
        // let mem2 = system.used_memory();
        // mem_usage = mem2 as f32 / total_mem2 as f32;
        mem_usage = 2.0;
    } else if server_num == 3 {
        // let total_mem3 = system.total_memory();
        // let mem3 = system.used_memory();
        // mem_usage = mem3 as f32 / total_mem3 as f32;
        mem_usage = 2.5;
    } else {
        mem_usage = 0.0;
    }

    // port 2222 server listen from server
    // port 8888 server send to server
    // port 3333 server listen from client
    // port 9999 server send to client

    let port0 = 2222;
    let port1 = 8888;
    let port2 = 3333;
    let port3 = 9999;

    let client_port = 5555; // client will be listening on this port

    let ports = vec![port0, port1, port2, port3];

    // socket for each server:port pair
    let socket1 = create_socket(server_ip, ports[0]); // server listen from server
    let socket2 = create_socket(server_ip, ports[1]); // server send to server
    let socket3 = create_socket(server_ip, ports[2]); // server listen from client
    let socket4 = create_socket(server_ip, ports[3]); // server send to client

    let client_data: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    // create a channel to communicate between the receiving thread and the main thread
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // spawn the thread that will receive image data from clients
    let data_arc = Arc::clone(&client_data);
    let tx_clone = mpsc::Sender::clone(&tx);
    let rec_socket = socket3.try_clone().unwrap();

    /////////////////////////////////////////////////////////////////
    /// thread to receive image data from clients
    thread::spawn(move || {
        let mut buffer = [0; 65535];
        let mut src_client;
        loop {
            // recieve a fragment from any client
            let (amt, src) = rec_socket
                .recv_from(&mut buffer)
                .expect("Didn't receive data");
            let recieved_chunk = &buffer[..amt];
            let sending_client = src.to_string();

            // add the fragment to the hashmap if client already sent, else create a new entry
            if data_arc.lock().unwrap().contains_key(&sending_client) {
                let mut x = data_arc.lock().unwrap();
                let mut temp = x.get_mut(&sending_client).unwrap();
                temp.append(&mut recieved_chunk.to_vec());
            } else {
                data_arc
                    .lock()
                    .unwrap()
                    .insert(sending_client, recieved_chunk.to_vec());
            }

            if recieved_chunk == b"MINSENDEND" {
                println!("Finished receiving image from client: {}", src.to_string());
                src_client = src.to_string();
                tx_clone.send(src_client).unwrap();
            }
        }
    });
    /////////////////////////////////////////////////////////////////
    // let mut buffer = [0; 65535];
    let mut leader: u16;
    let mut message_counter = 1;
    let mut election_starter = 1;
    let mut die_message_counter = 0;

    let default_image = file_as_dynamic_image("default.png".to_string());

    // send from server to another server
    thread::sleep(Duration::from_secs(3));
    loop {
        // starting election
        println!("----- MESSAGE NUMBER: {} ------", message_counter);
        leader = election_logic(
            server_num,
            mem_usage,
            election_starter,
            &servers,
            &ports,
            &socket1,
            &socket2,
        );

        //increase the memory usage for the leader
        if server_num == leader {
            mem_usage += 2.0;
            println!("** SERVER {} IS THE LEADER **", server_num);
        }

        //vector of bytes to store the image
        let mut src_client = rx.recv().unwrap();
        let mut image_from_client: Vec<u8> = Vec::new();

        //get the image from the hashmap with the client as the key using the get method
        // image_from_client = client_data.get(&src_client).unwrap().to_vec();
        image_from_client = client_data
            .lock()
            .unwrap()
            .get(&src_client)
            .unwrap()
            .to_vec();
        //remove it from the hashmap
        // client_data.remove(&src_client);
        client_data.lock().unwrap().remove(&src_client);

        // reconstruct the image from the fragments
        let mut reconstructed_image_bytes = Vec::new();
        for j in 0..image_from_client.len() {
            reconstructed_image_bytes.push(image_from_client[j]);
        }
        // let decoded_image = base64::decode(reconstructed_image_bytes).unwrap();
        let path = format!("decoded_image_message{}.png", message_counter);
        let mut file = File::create(path).unwrap();
        file.write_all(&reconstructed_image_bytes);

        // send from server to client
        if server_num == leader {
            // encode the recieved picture into the default picture
            let msg_bytes = &reconstructed_image_bytes.as_slice();
            let msg_bytes_base64 = base64::encode(msg_bytes);
            let bytes_to_send = msg_bytes_base64.as_bytes();
            let enc = Encoder::new(bytes_to_send, default_image.clone());
            let result = enc.encode_alpha();
            let path = format!("hidden_message_{}.png", message_counter);
            save_image_buffer(result, path.clone());

            // convert the result to base64
            let mut payload = File::open(path).unwrap();
            let mut payload_bytes = Vec::new();
            payload.read_to_end(&mut payload_bytes).unwrap();

            let mut fragmented_payload = Vec::new();
            for chunk in payload_bytes.chunks(1024) {
                fragmented_payload.push(chunk);
            }

            // send to client the encoded image.
            src_client = src_client.split(":").collect::<Vec<&str>>()[0].to_string();
            let temp = format!("{}:{}", src_client, ports[3]);

            for j in 0..fragmented_payload.len() {
                socket4
                    .send_to(fragmented_payload[j], &temp)
                    .expect("Failed to send data to client");
                // println!("Sent chunk {}", j);

                if j % 20 == 0 && j != 0 {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            socket4
                .send_to(b"MINSENDEND", &temp)
                .expect("Failed to send data to client");
            println!("Sent end to client: {}", src_client);
        }

        election_starter = leader;

        // revive the dead server by decreasing the memory usage of the dead server by 1000 after 3 messages
        if (mem_usage >= 1000.0) {
            let flag: bool = (message_counter != die_message_counter + 1)
                && (message_counter != die_message_counter + 2)
                && (message_counter != die_message_counter + 3);
            if (message_counter % 4 == 0) && (message_counter != 0) && flag {
                println!("----- RELOADING THIS SERVER -----");
                // change the memory usage of a random server
                mem_usage -= 1000.0;
            }
        }

        // simulate fault tolerance by increasing the memory usage of the leader server by 1000
        if (server_num == leader) {
            if (message_counter % 5 == 0) && (message_counter != 0) {
                println!("----- DROPPING THIS SERVER -----");
                // change the memory usage of a random server
                mem_usage += 1000.0;
                die_message_counter = message_counter;
            }
        }

        if (message_counter == 1) {
            thread::sleep(Duration::from_secs(1));
        }
        message_counter += 1;
    }
}
