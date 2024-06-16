use midir::{Ignore, MidiInput, MidiInputPort};
use rodio::{source::SineWave, OutputStream, Sink};
use std::collections::HashMap;
use std::error::Error;
use std::io::stdin;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the MIDI input
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get available MIDI input ports
    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
        println!("No available MIDI input ports.");
        return Ok(());
    }

    // Select the first available port
    let in_port: &MidiInputPort = &in_ports[0];

    println!(
        "Opening connection to port: {}",
        midi_in.port_name(in_port)?
    );

    // Initialize audio output
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let mut synth = Synthesizer::new(stream_handle);

    // Define a callback to handle incoming MIDI messages
    let in_port_name = midi_in.port_name(in_port)?;
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            decode_midi_message(&mut synth, message);
        },
        (),
    )?;

    println!(
        "Connection open, reading MIDI input from '{}'. Press Enter to exit...",
        in_port_name
    );

    // Wait for user input to exit
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    println!("Closing connection");
    Ok(())
}

fn decode_midi_message(synth: &mut Synthesizer, message: &[u8]) {
    if message.is_empty() {
        return;
    }

    match message[0] {
        0x80..=0x8F => {
            println!(
                "Note Off: channel={}, note={}, velocity={}",
                message[0] & 0x0F,
                message[1],
                message[2]
            );
            synth.note_off(message[1]);
        }
        0x90..=0x9F => {
            println!(
                "Note On: channel={}, note={}, velocity={}",
                message[0] & 0x0F,
                message[1],
                message[2]
            );
            synth.note_on(message[1], message[2]);
        }
        0xA0..=0xAF => println!(
            "Polyphonic Key Pressure: channel={}, note={}, pressure={}",
            message[0] & 0x0F,
            message[1],
            message[2]
        ),
        0xB0..=0xBF => println!(
            "Control Change: channel={}, controller={}, value={}",
            message[0] & 0x0F,
            message[1],
            message[2]
        ),
        0xC0..=0xCF => println!(
            "Program Change: channel={}, program={}",
            message[0] & 0x0F,
            message[1]
        ),
        0xD0..=0xDF => println!(
            "Channel Pressure: channel={}, pressure={}",
            message[0] & 0x0F,
            message[1]
        ),
        0xE0..=0xEF => println!(
            "Pitch Bend Change: channel={}, lsb={}, msb={}",
            message[0] & 0x0F,
            message[1],
            message[2]
        ),
        _ => println!("Unknown message: {:?}", message),
    }
}

struct Synthesizer {
    stream_handle: rodio::OutputStreamHandle,
    sinks: HashMap<u8, Sink>,
}

impl Synthesizer {
    fn new(stream_handle: rodio::OutputStreamHandle) -> Self {
        Synthesizer {
            stream_handle,
            sinks: HashMap::new(),
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        let frequency = midi_note_to_freq(note);
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(SineWave::new(frequency));
        self.sinks.insert(note, sink);
    }

    fn note_off(&mut self, note: u8) {
        if let Some(sink) = self.sinks.remove(&note) {
            sink.stop();
        }
    }
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * (2.0f32).powf((note as f32 - 69.0) / 12.0)
}
