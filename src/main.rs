//! San Diego Rust
//! April 2018 Challenge
//!
//! Controlling an audio mixer using OSC over UDP.
//!
//! See README.md for challenge details.
#![allow(unused_imports)]

use std::str;

use std::io::{Read, Write};
use std::net::UdpSocket;

// The byteorder crate makes it easier to read/write binary numeric values.
extern crate byteorder;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};

// Change this to match the IP address of the XR-12 mixer.
// The mixer accepts OSC commands on UDP port 10024.
const MIXER_ADDR: &str = "192.168.1.181:10024";

fn pad(buf: &mut Vec<u8>) {
    let zeros: &[u8] = &[0; 3];
    let m = buf.len() % 4;
    if m != 0 {
        buf.extend(&zeros[..4 - m]);
    }
}

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

#[derive(Debug)]
struct Command {
    address_pattern: String,
    arguments: Vec<Argument>,
}

impl Command {
    fn encode(&self, buf: &mut Vec<u8>) {
        // Encode the address pattern.
        buf.extend(self.address_pattern.as_bytes());
        buf.push(0);
        pad(buf);

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

    fn decode(buf: &[u8]) -> Command {
        let (address_pattern, buf) = decode_string(buf);

        let (type_tags, mut buf) = decode_string(buf);
        assert_eq!(",", &type_tags[..1]);
        let type_tags = &type_tags[1..];

        let mut arguments = Vec::with_capacity(type_tags.len());
        for type_tag in type_tags.chars() {
            match type_tag {
                's' => {
                    let (s, b) = decode_string(buf);
                    arguments.push(Argument::String(s.to_string()));
                    buf = b;
                },
                'i' => {
                    arguments.push(Argument::Integer(BigEndian::read_i32(&buf[..4])));
                    buf = &buf[4..];
                },
                'f' => {
                    arguments.push(Argument::Float(BigEndian::read_f32(&buf[..4])));
                    buf = &buf[4..];
                },
                'b' => {
                    let len = BigEndian::read_u32(&buf[..4]) as usize;
                    arguments.push(Argument::Binary(buf[4..][..len].to_owned()));
                    buf = &buf[4 + len..];
                },
                _ => panic!("unknown type tag: {}", type_tag),
            }
        }
        Command {
            address_pattern: address_pattern.to_string(),
            arguments,
        }
    }
}

#[derive(Debug)]
enum Argument {
    String(String),
    Integer(i32),
    Float(f32),
    Binary(Vec<u8>),
}

impl Argument {
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

    // Send the command.
    let command = Command {
        address_pattern: "/info".to_string(),
        arguments: vec![],
    };
    let mut buf = Vec::new();
    command.encode(&mut buf);
    println!("request: {:?}", command);
    socket.send(&buf).expect("failed to write to UDP socket");

    // Receive the response.
    let mut buf = [0; 4096];
    let len = socket.recv(&mut buf).expect("failed to receive response");
    println!("response: {:?}", Command::decode(&buf[..len]));

    /*
    // Challenge 2.

    let command = Command {
        address_pattern: "/ch/01/mix/fader".to_string(),
        arguments: vec![Argument::Float(0.9)],
    };
    let mut buf = Vec::new();
    command.encode(&mut buf);
    println!("request: {:?}", command);
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

    let asterisks = [b'*'; 64];
    let mut buf = [0; 4096 * 4];
    loop {
        let len = socket.recv(&mut buf).expect("failed to receive response");
        let command = Command::decode(&buf[..len]);
        println!("response: {:?}", command);

        let binary = if let Argument::Binary(ref b) = command.arguments[0] {
            b
        } else {
            panic!("not a binary argument");
        };

        let channel1 = LittleEndian::read_i16(&binary[4..]);

        let width = ((32767i16 + channel1) as f64).log2() as usize;
        println!("width: {}, channel1: {}", width, channel1);

        let asterisks = unsafe { str::from_utf8_unchecked(&asterisks) };

        println!("{}", &asterisks[..width]);

    }
    */
}
