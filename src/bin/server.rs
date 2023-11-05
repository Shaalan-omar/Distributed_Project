use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, SocketAddr, IpAddr};
use std::time::Duration;
use sysinfo::{System, SystemExt};
use std::{process, str, thread};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    server: u16,
    mem_usage: f32,
}

fn create_socket(server_ip: &str, port: u16) -> UdpSocket {
    let server_address = format!("{}:{}", server_ip, port);
    let socket_addr: SocketAddr = server_address.parse().expect("Failed to parse socket address");
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

    // send from server to another server
    thread::sleep(Duration::from_secs(3));

    let mut buffer = [0; 16384];
    let mut leader: u16;
    let mut message_counter = 1;
    let mut election_starter = 1;
    let mut die_message_counter = 0;
    loop {
        println!("----- MESSAGE NUMBER: {} ------", message_counter);
        // println!("Memory usage: {}", mem_usage);
        // println!("STARTED ELECTION");
        leader = election_logic(
            server_num, mem_usage, election_starter, &servers, &ports, &socket1, &socket2,
        );
        // println!("FINISHED ELECTION");
        // println!("leader: {}", leader);

        //increase the memory usage for the leader
        if server_num == leader {
            mem_usage += 2.0;
            println!("** SERVER {} IS THE LEADER **", server_num);
        }

        //all servers listen from client
        let (amt, src) = socket3.recv_from(&mut buffer).expect("Didn't receive data");
        let mut msg = str::from_utf8(&buffer[..amt]).unwrap();
        println!("Received: {} from {}", msg, src);
        let mut src_client = src.to_string();
        //remove the message from the buffer
        buffer = [0; 16384];

        // play with the message here

        // send from server to client
        if server_num == leader {
            src_client = src_client.split(":").collect::<Vec<&str>>()[0].to_string();
            let temp = format!("{}:{}", src_client, ports[3]);
            socket4
                .send_to("Hello client".as_bytes(), &temp)
                .expect("Failed to send data to client");
            println!(
                "server {} sent to client with address {}",
                server_num, src_client
            );
        }

        election_starter = leader;

        // revive the dead server by decreasing the memory usage of the dead server by 1000 after 3 messages
        if(mem_usage >= 1000.0){
            let flag:bool = (message_counter != die_message_counter + 1) && (message_counter != die_message_counter + 2) && (message_counter != die_message_counter + 3);
            if (message_counter % 4 == 0) && (message_counter != 0) && flag{
                println!("----- RELOADING THIS SERVER -----");
                // change the memory usage of a random server
                mem_usage -= 1000.0;
            }
        }

        // simulate fault tolerance by increasing the memory usage of the leader server by 1000
        if(server_num == leader){
            if (message_counter % 5 == 0) && (message_counter != 0){
                println!("----- DROPPING THIS SERVER -----");
                // change the memory usage of a random server
                mem_usage += 1000.0;
                die_message_counter = message_counter;
            }
        }

        if (message_counter == 1){
            thread::sleep(Duration::from_secs(1));
        }
        message_counter +=1;
    }
}





// 
// fn main() {
    // let port1: u16 = std::env::args()
        // .nth(1)
        // .expect("no port number provided")
        // .parse()
        // .unwrap();
    // let port2: u16 = std::env::args()
        // .nth(2)
        // .expect("no other port number provided")
        // .parse()
        // .unwrap();
    // let port3: u16 = std::env::args()
        // .nth(3)
        // .expect("no other port number provided")
        // .parse()
        // .unwrap();
    // let port: u16 = std::env::args()
        // .nth(4)
        // .expect("didn't specify which port")
        // .parse()
        // .unwrap();
// 
    // let ports = vec![port1, port2, port3];
// 
//     Create a socket for the server.
//     // let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
// // 
//     to allow multiple instances to bind to the same port.
//     // socket.set_reuse_address(true).unwrap();
// // 
//     // let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), ports[port as usize - 1]);
//     // socket.bind(&SockAddr::from(addr)).unwrap();
// // 
//     // socket
//         // .join_multicast_v4(&Ipv4Addr::new(239, 0, 0, 1), &Ipv4Addr::new(0, 0, 0, 0))
//         // .unwrap();
// // 
//     Convert the socket back to a UdpSocket.
//     // let socket = UdpSocket::from(socket);
// // 
//     // let pid = process::id();
// // 
//     get the memory usage per server
    // let mut system = System::new_all();
    // system.refresh_all();
// 
    // let mut mem_usage:f32;
// 
    // if port == 1{
    //     let total_mem1 = system.total_memory();
    //     let mem1 = system.used_memory();
    //     mem_usage = mem1 as f32 / total_mem1 as f32;
    //     // mem_usage = 1.0;
    // // } else if port == 2{
    //     let total_mem2 = system.total_memory();
    //     let mem2 = system.used_memory();
    //     mem_usage = mem2 as f32 / total_mem2 as f32;
    //     // mem_usage = 3.0;
    // // } else if port == 3{
    //     let total_mem3 = system.total_memory();
    //     let mem3 = system.used_memory();
    //     mem_usage = mem3 as f32 / total_mem3 as f32;
        // mem_usage = 2.5;
    // } else{
        // mem_usage = 0.0;
    // }
    // println!("mem usage: {} for port: {}", mem_usage, port);
// 
    // let server1 = format!("127.0.0.1:{}", port1);
    // let server2 = format!("127.0.0.1:{}", port2);
    // let server3 = format!("127.0.0.1:{}", port3);
// 
    // to store the incoming messages.
    // let mut buffer = [0; 1024];
    // let mut is_leader = true;
    // let mut winner: u16;
    // let mut delay_flag = false;
    // 
    // now listen to client and respond if you are the leader
    // loop {
        // let (amt, src) = socket.recv_from(&mut client_buffer).expect("Didn't receive data");
        // let msg = str::from_utf8(&client_buffer[..amt]).unwrap();
        // println!("Received: {} from {}", msg, src);
// 
        // winner = election(delay_flag, port1, port2, port3, port, ports.clone(), mem_usage, &socket, server1.clone(), server2.clone(), server3.clone(), pid, buffer, &is_leader);
        // println!("Value of is_leader: {} at port: {}", is_leader, port);
        // if (delay_flag == false){
            // delay_flag = true;
        // }
// 
        // if port == winner {
            // let response = format!(
                // "{}: response from server listening on port {} with PID: {}",
                // msg, port, pid
            // );
            // socket
                // .send_to(response.as_bytes(), &src)
                // .expect("failed to send response");
            // mem_usage += 3.0;
        // }
// 
        // empty the buffer
        // buffer = [0; 1024];
    // }
// 
// }
// 
// fn election(
    // delay_flag: bool,
    // port1: u16,
    // port2: u16,
    // port3: u16, 
    // port: u16, 
    // ports: Vec<u16>, 
    // mut mem_usage:f32, 
    // mut socket: &UdpSocket, 
    // server1: String, 
    // server2: String, 
    // server3: String, 
    // pid: u32, 
    // mut buffer: [u8; 1024],
    // mut is_leader: &bool,
// ) -> u16{
// 
    // let servers = vec![server1, server2, server3];
// 
    // let server_info = ServerInfo {
        // port: port,
        // mem_usage: mem_usage,
    // };
// 
    // let server_info_str = serde_json::to_string(&server_info).unwrap();
// 
    // Initially, each server assumes it's the leader.
    // println!(
        // "the server listening on port {} has set itself as the leader",
        // ports[port as usize - 1]
    // );
// 
    // if(delay_flag == false){
        // thread::sleep(Duration::from_secs(3));
    // }
// 
    // if port == 1 {
    //     sent port number to server listening on port two to check who is listening on a lower port
    //     // socket
    //         // .send_to(server_info_str.as_bytes(), &servers[2 - 1])
    //         // .expect("Failed to send initial leader claim");
    // // } else if port == 2 {
    //     receive port number from server listening on port 1, then compare who has the lower port and send the result to port 3
    //     // let (amt, src) = socket.recv_from(&mut buffer).expect("Didn't receive data");
    //     // let mut msg: &str;
    //     i want to receive from the server, not the client
    //     if(src.to_string() != servers[0]){
    //         println!("This is from a client: {}", src);
            //write to buffer client
            // msg = str::from_utf8(&buffer[..amt]).unwrap();
            // remove from buffer and add to client buffer
            // client_buffer = buffer;
            // buffer = &[0; 1024];
        // }else{
            // msg = str::from_utf8(&buffer[..amt]).unwrap();
            // println!("Received: {} from {}", msg, src);
        // 
        // let port1_claim_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // let port1_claim = port1_claim_info.mem_usage;
        // println!("port1 claim: {}", port1_claim);
        // if port1_claim <= mem_usage {
            // socket
                // .send_to(msg.to_string().as_bytes(), &servers[3 - 1])
                // .expect("Failed to send initial leader claim");
            // println!("sent {} to server listening on port {}", msg, port3);
            // is_leader = &false;
        // } else {
            // socket
                // .send_to(server_info_str.as_bytes(), &servers[3 - 1])
                // .expect("Failed to send initial leader claim");
            // println!("sent {} to server listening on port {}", server_info_str, port3);
        // }
    // } else if port == 3 {
        // receive port number from server listening on port 2, then compare who has the lower port and send the result to the two other servers
        // // let (amt, src) = socket.recv_from(&mut buffer).expect("Didn't receive data");
        // // let mut msg: &str;
        // if(src.to_string() != servers[1]){
        //     println!("This is from a client: {}", src);
        //     write to buffer client
        //     msg = str::from_utf8(&buffer[..amt]).unwrap();
        //     remove from buffer and add to client buffer
        //     client_buffer = buffer;
        //     buffer = &[0; 1024];
        // }else{
            // msg = str::from_utf8(&buffer[..amt]).unwrap();
            // println!("Received: {} from {}", msg, src);
        // 
        // let port2_claim_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // let port2_claim = port2_claim_info.mem_usage;
        // println!("port2 claim: {}", port2_claim);
        // if port2_claim <= mem_usage {
            // socket
                // .send_to(msg.to_string().as_bytes(), &servers[1 - 1])
                // .expect("Failed to send initial leader claim");
            // socket
                // .send_to(msg.to_string().as_bytes(), &servers[2 - 1])
                // .expect("Failed to send initial leader claim");
            // is_leader = &false;
            // println!("sent {} to server listening on port {}", msg, port1);
            // println!("sent {} to server listening on port {}", msg, port2);
            // return port2_claim_info.port;
        // } else {
            // socket
                // .send_to(server_info_str.as_bytes(), &servers[1 - 1])
                // .expect("Failed to send initial leader claim");
            // socket
                // .send_to(server_info_str.as_bytes(), &servers[2 - 1])
                // .expect("Failed to send initial leader claim");
            // println!("sent {} to server listening on port {}", server_info_str, port1);
            // println!("sent {} to server listening on port {}", server_info_str, port2);
            // return port;
        // }
    // 
    // }
// 
    // by now, all three servers know which server is listening to the lowest port.
    // server 2 and 3 will receive the message from the server listening on port 3 to know who the leader is.
    // if port == 1 || port == 2 {
        // let (amt, src) = socket.recv_from(&mut buffer).expect("Didn't receive data");
        // let mut msg: &str;
        // if(src.to_string() != servers[2]){
            // println!("This is from a client: {}", src);
            //write to buffer client
            // msg = str::from_utf8(&buffer[..amt]).unwrap();
            // remove from buffer and add to client buffer
            // client_buffer = buffer;
            // buffer = &[0; 1024];
        // }else{
            // msg = str::from_utf8(&buffer[..amt]).unwrap();
            // println!("Received: {} from {}", msg, src);
        // 
        // println!("I RECIEVED THIS: {} from {}", msg, src);
        // let winner_info: ServerInfo = serde_json::from_str(msg).unwrap();
        // let winner = winner_info.port;
        // println!("winner: {}", winner);
        // if winner == port {
            // println!(
                // "server {} has won, it is now the actual leader",
                // servers[port as usize - 1]
            // );
        // } else {    
            // is_leader = &false;
            // println!(
                // "server {} has set {} as the leader",
                // servers[port as usize - 1], winner
            // );
        // }
        // return winner;
    // }
    // return 0;
// }
// 
