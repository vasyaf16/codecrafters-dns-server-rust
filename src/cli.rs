use std::net::SocketAddrV4;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short = 'r', long)]
    pub resolver: Option<SocketAddrV4>
}