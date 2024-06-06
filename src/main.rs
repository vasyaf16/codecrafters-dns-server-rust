// Uncomment this block to pass the first stage
use std::net::UdpSocket;

mod header;
mod question;
mod message;
mod answer;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let message = message::Message::deserialize(&buf[..size]);
                let (id, opcode, rd ) = (message.id(), message.opcode(), message.rd());
                let m = message::MessageBuilder::new()
                    .set_id(id)
                    .set_opcode(opcode)
                    .set_rd(rd)
                    .add_answers(message.answers)
                    .add_questions(message.questions)
                    .finish();
                let response = m.serialize();
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
