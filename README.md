
# San Diego Rust April 2018 Challenge
Write a Rust app to control the Behringer XR-12 audio mixer! The XR-12 is a WiFi-enabled audio mixer that can be controlled using Open Sound Control (OSC) over UDP.

## OSC quick description
OSC is a simple binary messaging protocol used by many audio devices. Here is the hex dump of an example OSC message to call a method "/play/note" with the arguments "Guitar", 12, and 1.0:

```
2f706c61792f6e6f74650000  2c73696600000000  4775697461720000  12000000  3f800000
|-> /play/note            |-> ,sif          |-> Guitar        |-> 12    |-> 1.0
```

 * ```/play/note``` is the _address pattern_,  a URI-like string that identifies the method to execute.
 * ```,sif``` is  the _type tag_,  a string describing the types of the method's parameters. The type tag starts with a `,` and each following letter indicates the type of one parameter. `,sif` describes 3 parameters:
	 * `s`: a null-terminated string
	 * `i`: a 32-bit signed integer
	 * `f`: a 32-bit float
	 * An OSC message with zero parameters should still have a type tag (a comma followed by three zero bytes).
 * The _parameters_ follow the type tag. Note that all strings must be null-terminated, and all numbers are stored in binary big-endian format.
 * __All OSC fields must be aligned to 4 bytes__. Extra zero bytes must be inserted to pad each field to a multiple of 4 bytes. For example, the command string `/play/note` has two zero bytes on the end to pad it to a length of 12 bytes.

Consult the [OSC specifications](http://opensoundcontrol.org/spec-1_0) for more detailed information.

## The challenges!
1) Create a `UdpSocket` and connect to the mixer on UDP port 10024. Send the `/info` command to the mixer, and inspect the response. The mixer will return an OSC response with version info about the mixer. __Remember that all fields in an OSC command must be aligned to 4 bytes.__ `/info` is 6 bytes (including the trailing null) and will require an extra 2 null bytes to pad it to 8 bytes.

2) An audio input is connected to channel 1 of the mixer, but channel 1's volume is turned all the way down. The OSC command  `/ch/01/mix/fader` can be used to control the channel's fader (volume knob). It takes a single float parameter between 0.0 to 1.0. You will hear some audio if you are succesful.

3) __Bonus Hard Challenge:__ Display an audio level meter for Channel 1. The meter represents the loudness of the music as it's playing.
	 * Use the `/meters.,s../meters/1` command to request periodic updates for the about audio levels for the next 10 seconds (`.` represents a zero byte). You will receive OSC commands from the mixer containing updates in this format:
	 * `/meters/1...,b..<binary blob>`
	 * The binary blob is in this format:
		 * Length of blob (32-bit big endian)
		 * The number of volume values in the blob (32-bit _little endian_)
		 * An array of 16-bit values (16-bit _little endian_)
		 * Read the first 16-bit value because it represents Channel 1; you can ignore the rest.
		 * The value is an integer, usually in the range of -32768 to 0. 0 reprsents max volume (clipping).
	 * Each time you receive an `meters` message, output a row of asterisks based on the loudness of the audio.
  
## Helpful docs + links

*  [Rust UdpSocket docs](https://doc.rust-lang.org/std/net/struct.UdpSocket.html)
*  [Rust byteorder crate](https://github.com/BurntSushi/byteorder), useful for writing binary data
*  [Open Sound Control 1.0 specification](http://opensoundcontrol.org/spec-1_0)
*  [Behringer X-series OSC protocol](https://bf95dc13-a-62cb3a1a-s-sites.googlegroups.com/site/patrickmaillot/docs/X32-OSC.pdf?attachauth=ANoY7crX7fTjAD43lfQPjOj6RktL5TNWInxa8pFcjGVXeKRdbWVIQgh1Hy7R52diMmJcdjx3obDEw2gIBpYNFAiP21oxqupRuMcjjwKd6K9Je1KVarCdYSXlOvVOsIfN-DaVZMV9xrmhSBThPuS4uFsjiJg5l1C9U9dEr0cFdcLGxAVG89y6F9cypLT1pNplmxC-olzQ3_4gQn2br7Bv3SY1b81ZJIUnTA%3D%3D&attredirects=0)