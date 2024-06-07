// Uncomment this block to pass the first stage
use std::net::{ SocketAddrV4, UdpSocket};
use clap::Parser;
use crate::cli::Args;
use crate::message::{Answers, Message};

mod header;
mod question;
mod message;
mod answer;
mod cli;

fn forwarding_server(udp_socket: &UdpSocket, message: Message, socket_addr: SocketAddrV4) -> Message {
    Message::join(message.split().into_iter().map(|m|
        {
            println!("{:?}", m);
            let s = m.serialize();
            udp_socket.send_to(&s, socket_addr).expect("Failed to send response");
            let mut buf = [0; 512];
            let (n, _) = udp_socket.recv_from(&mut buf).expect("Failed to recv response");
            Message::deserialize(&buf[..n])
        }
    ).collect::<Vec<_>>())

}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let args = Args::parse();
    let is_forwarded_server = args.resolver.is_some();

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let message = Message::deserialize(&buf[..size]);
                println!("Recvd message : {:?}", message);
                let response = if is_forwarded_server {
                    let m = forwarding_server(&udp_socket, message, args.resolver.unwrap());
                    println!("response message : {:?}", m);
                    m.serialize()
                } else {
                    let (id, opcode, rd) = (message.id(), message.opcode(), message.rd());
                    let m = message::MessageBuilder::new()
                        .set_id(id)
                        .set_opcode(opcode)
                        .set_rd(rd)
                        .add_answers(Answers::from_questions(&message.questions))
                        .add_questions(message.questions)
                        .finish();
                    let response = m.serialize();
                    response
                };
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
