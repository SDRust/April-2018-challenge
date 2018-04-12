//! San Diego Rust
//! April 2018 Challenge
//! 
//! Controlling an audio mixer using OSC over UDP.
//! 
//! See README.md for challenge details.
#![allow(unused_imports)]

use std::io::{Read, Write};
use std::net::UdpSocket;

// The byteorder crate makes it easier to read/write binary numeric values.
extern crate byteorder;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

// Change this to match the IP address of the XR-12 mixer.
// The mixer accepts OSC commands on UDP port 10024.
const MIXER_ADDR: &str = "192.168.1.1:10024";

fn main() {
    // TODO: Create a UDP socket and send an OSC command to MIXER_ADDR.
}
