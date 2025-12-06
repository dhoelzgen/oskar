use crate::hid::MidiCC;

pub struct MidiConfig {
    pub channel: u8,
}

pub struct KeyLayout {
    pub encoder_left: MidiCC,
    pub encoder_right: MidiCC,
    pub encoder_button: MidiCC,
    pub key1: MidiCC,
    pub key2: MidiCC,
    pub key3: MidiCC,
}