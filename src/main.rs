//! San Diego Rust
//! April 2018 Challenge
//!
//! Controlling an audio mixer using OSC over UDP.
//!
//! See README.md for challenge details.

#![allow(unused_imports)]

extern crate byteorder;

use std::net::UdpSocket;
use std::str;
use std::i16;

use byteorder::{
    BigEndian,
    ByteOrder,
    LittleEndian,
    WriteBytesExt,
};

// Change this to match the IP address of the XR-12 mixer.
// The mixer accepts OSC commands on UDP port 10024.
const MIXER_ADDR: &str = "192.168.1.181:10024";

/// Pads the provided buffer with null bytes to be 4-byte aligned.
fn pad(buf: &mut Vec<u8>) {
    let zeros: &[u8] = &[0; 3];
    let m = buf.len() % 4;
    if m != 0 {
        buf.extend(&zeros[..4 - m]);
    }
}

/// Encodes the provided string to the buffer.
fn encode_string(s: &str, buf: &mut Vec<u8>) {
    buf.extend(s.as_bytes());
    buf.push(0);
    pad(buf);
}

/// Decodes a string from the buffer, returning the string value and the remaining buffer.
fn decode_string(buf: &[u8]) -> (&str, &[u8]) {
    let idx = buf.iter().position(|&x| x == 0).expect("no null terminator");
    let s = str::from_utf8(&buf[..idx]).expect("not valid UTF-8");

    // unpad
    let m = idx % 4;
    let buf = if m == 0 {
        &buf[idx..]
    } else {
        &buf[idx + 4 - m..]
    };

    (s, buf)
}

/// An OSC command.
#[derive(Debug)]
pub struct Command {
    address_pattern: String,
    arguments: Vec<Argument>,
}

impl Command {

    /// Encodes the command to a buffer.
    fn encode(&self, buf: &mut Vec<u8>) {
        // Encode the address pattern.
        encode_string(&self.address_pattern, buf);

        // Encode the type tags.
        buf.push(b',');
        for argument in &self.arguments {
            buf.push(argument.tag());
        }
        buf.push(0);
        pad(buf);

        // Encode the arguments.
        for argument in &self.arguments {
            argument.encode(buf);
            pad(buf);
        }
    }

    /// Decodes a Command from a buffer.
    fn decode(buf: &[u8]) -> Command {
        let (address_pattern, buf) = decode_string(buf);

        let (type_tags, mut buf) = decode_string(buf);
        assert_eq!(",", &type_tags[..1]);
        let type_tags = &type_tags[1..];

        let mut arguments = Vec::with_capacity(type_tags.len());
        for type_tag in type_tags.chars() {
            let (a, b) = Argument::decode(type_tag, buf);
            arguments.push(a);
            buf = b;
        }
        Command {
            address_pattern: address_pattern.to_string(),
            arguments,
        }
    }
}

/// An OSC Command argument.
#[derive(Debug)]
pub enum Argument {
    String(String),
    Integer(i32),
    Float(f32),
    Binary(Vec<u8>),
}

impl Argument {

    /// Encode the argument to a buffer.
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            Argument::String(s) => {
                buf.extend(s.as_bytes());
                buf.push(0);
            },
            Argument::Integer(i) => buf.write_i32::<BigEndian>(*i).unwrap(),
            Argument::Float(f) => buf.write_f32::<BigEndian>(*f).unwrap(),
            Argument::Binary(b) => {
                buf.write_u32::<BigEndian>(b.len() as u32).unwrap();
                buf.extend(b);
            },
        }
    }

    /// Decodes an argument of the provided type from a buffer, returning the argument, and the
    /// remaining buffer.
    fn decode(type_tag: char, buf: &[u8]) -> (Argument, &[u8]) {
        match type_tag {
            's' => {
                let (s, b) = decode_string(buf);
                (Argument::String(s.to_string()), b)
            },
            'i' => (Argument::Integer(BigEndian::read_i32(&buf[..4])), &buf[4..]),
            'f' => (Argument::Float(BigEndian::read_f32(&buf[..4])), &buf[4..]),
            'b' => {
                let len = BigEndian::read_u32(&buf[..4]) as usize;
                (Argument::Binary(buf[4..][..len].to_owned()), &buf[4 + len..])
            },
            _ => panic!("unknown type tag: {}", type_tag),
        }
    }

    /// Returns the tag byte for the argument type.
    fn tag(&self) -> u8 {
        match self {
            Argument::String(_) => b's',
            Argument::Integer(_) => b'i',
            Argument::Float(_) => b'f',
            Argument::Binary(_) => b'b',
        }
    }
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
    socket.connect(MIXER_ADDR).expect("failed to connect to mixer");

    // Challenge 1.

    // Send the request.
    let request = Command {
        address_pattern: "/info".to_string(),
        arguments: vec![],
    };
    let mut buf = Vec::new();
    request.encode(&mut buf);
    println!("request: {:?}", request);
    socket.send(&buf).expect("failed to write to UDP socket");

    // Receive the response.
    let mut buf = [0; 4096];
    let len = socket.recv(&mut buf).expect("failed to receive response");
    println!("response: {:?}", Command::decode(&buf[..len]));

    // Challenge 2.

    let request = Command {
        address_pattern: "/ch/01/mix/fader".to_string(),
        arguments: vec![Argument::Float(0.0)],
    };
    let mut buf = Vec::new();
    request.encode(&mut buf);
    println!("request: {:?}", request);
    socket.send(&buf).expect("failed to write to UDP socket");

    // Bonus!

    let request = Command {
        address_pattern: "/meters".to_string(),
        arguments: vec![Argument::String("/meters/1".to_string())],
    };
    let mut buf = Vec::new();
    request.encode(&mut buf);
    println!("request: {:?}", request);
    socket.send(&buf).expect("failed to write to UDP socket");

    let asterisks = str::from_utf8(&[b'*'; 1024]).unwrap();
    let mut buf = [0; 4096 * 4];
    loop {
        let len = socket.recv(&mut buf).expect("failed to receive response");
        let response = Command::decode(&buf[..len]);
        //println!("response: {:?}", response);

        let binary = if let Argument::Binary(ref b) = response.arguments[0] {
            b
        } else {
            panic!("not a binary argument");
        };

        // Channel 1 comes after another 32 bit length field, so offset by 4.
        let channel1 = LittleEndian::read_i16(&binary[4..]);

        let width = ((i16::MAX + channel1) as f64) as usize / 256 ;
        println!("channel 1: {}\t{}", channel1, &asterisks[..width]);
    }
}
