use crate::{EncoderResources, ButtonResources};
use crate::layouts::{KeyLayout, MidiConfig};
use defmt::unreachable;
use defmt_rtt as _;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::select::select_array;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::interrupt;
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::midi::MidiClass;
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

type CustomMidi = MidiClass<'static, Driver<'static, USB>>;
static KEY_EVENT_QUEUE: PubSubChannel::<CriticalSectionRawMutex, KeyEvent, 2, 2, 2> = PubSubChannel::new();

#[derive(Clone)]
#[derive(PartialEq)]
enum Key {
    EncoderLeft,
    EncoderRight,
    EncoderButton,
    Key1,
    Key2,
    Key3,
}

#[derive(Clone)]
#[derive(PartialEq)]
enum Event {
    Pressed,
    Released,
}

#[derive(Clone)]
struct KeyEvent {
    key: Key,
    event: Event,
}

pub struct MidiCC {
    pub controller: u8,
    pub value_on: u8,
    pub value_off: u8,
}

// MIDI Configuration: Channel 15 (0-indexed as 14), configurable CC numbers
const MIDI_CONFIG: MidiConfig = MidiConfig {
    channel: 14, // MIDI channel 15 (0-indexed)
};

const KEYLAYOUT: KeyLayout = KeyLayout {
    encoder_left: MidiCC { controller: 1, value_on: 127, value_off: 0 },    // CC 1 (Modulation)
    encoder_right: MidiCC { controller: 2, value_on: 127, value_off: 0 },   // CC 2 (Breath Controller)
    encoder_button: MidiCC { controller: 3, value_on: 127, value_off: 0 },  // CC 3
    key1: MidiCC { controller: 20, value_on: 127, value_off: 0 },           // CC 20
    key2: MidiCC { controller: 21, value_on: 127, value_off: 0 },           // CC 21
    key3: MidiCC { controller: 22, value_on: 127, value_off: 0 },           // CC 22
};

#[embassy_executor::task]
pub async fn hid_task(spawner: Spawner, mut midi_class: CustomMidi, button_resources: ButtonResources, encoder_resources: EncoderResources) -> ! {

    interrupt::SWI_IRQ_0.set_priority(Priority::P2);
    let spawner_encoder: embassy_executor::SendSpawner = EXECUTOR_ENCODER.start(interrupt::SWI_IRQ_0);
    spawner_encoder.spawn(encoder_task(encoder_resources)).unwrap();

    spawner.spawn(button_task(button_resources)).unwrap();

    let mut sub = KEY_EVENT_QUEUE.subscriber().unwrap();

    loop {
        let key_event: KeyEvent = sub.next_message_pure().await;

        match key_event.key {
            Key::EncoderLeft => {
                midi_class = handle_encoder_interaction(midi_class, KEYLAYOUT.encoder_left).await;
            },
            Key::EncoderRight => {
                midi_class = handle_encoder_interaction(midi_class, KEYLAYOUT.encoder_right).await;
            },
            Key::EncoderButton => {
                midi_class = send_midi_cc(midi_class, KEYLAYOUT.encoder_button, key_event.event).await;
            },
            Key::Key1 => {
                midi_class = send_midi_cc(midi_class, KEYLAYOUT.key1, key_event.event).await;
            },
            Key::Key2 => {
                midi_class = send_midi_cc(midi_class, KEYLAYOUT.key2, key_event.event).await;
            },
            Key::Key3 => {
                midi_class = send_midi_cc(midi_class, KEYLAYOUT.key3, key_event.event).await;
            }
        }
    }
}


static EXECUTOR_ENCODER: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn SWI_IRQ_0() {
    unsafe { EXECUTOR_ENCODER.on_interrupt() }
}

#[embassy_executor::task]
pub async fn encoder_task(r: EncoderResources) -> ! {

    let encoder_left: Input<'_> = Input::new(r.encoder_left, Pull::None);

    let mut encoder_right: Input<'_> = Input::new(r.encoder_right, Pull::None);

    let publisher = KEY_EVENT_QUEUE.publisher().unwrap();

    loop {
        encoder_right.wait_for_falling_edge().await;

        if encoder_left.get_level() == Level::Low {
            publisher.publish_immediate(KeyEvent {key: Key::EncoderLeft, event: Event::Pressed});
        } else {
            publisher.publish_immediate(KeyEvent {key: Key::EncoderRight, event: Event::Pressed});
        };

        encoder_right.wait_for_rising_edge().await;
    }
}

#[embassy_executor::task]
pub async fn button_task(r: ButtonResources) -> ! {

    let mut key1: Input<'_> = Input::new(r.key1, Pull::Up);
    key1.set_schmitt(true);

    let mut key2: Input<'_> = Input::new(r.key2, Pull::Up);
    key2.set_schmitt(true);

    let mut key3: Input<'_> = Input::new(r.key3, Pull::Up);
    key3.set_schmitt(true);

    let mut encoder_button: Input<'_> = Input::new(r.encoder_button, Pull::Up);
    encoder_button.set_schmitt(true);

    let publisher = KEY_EVENT_QUEUE.publisher().unwrap();

    loop {

        let (_, index) = select_array([
            key1.wait_for_any_edge(),
            key2.wait_for_any_edge(),
            key3.wait_for_any_edge(),
            encoder_button.wait_for_any_edge(),
        ])
        .await;

        match index {
            0 => {
                match key1.get_level() {
                    Level::Low => publisher.publish_immediate(KeyEvent {key: Key::Key1, event: Event::Pressed}),
                    Level::High => publisher.publish_immediate(KeyEvent {key: Key::Key1, event: Event::Released}),
                }
            }
            1 => {
                match key2.get_level() {
                    Level::Low => publisher.publish_immediate(KeyEvent {key: Key::Key2, event: Event::Pressed}),
                    Level::High => publisher.publish_immediate(KeyEvent {key: Key::Key2, event: Event::Released}),
                }
            }
            2 => {
                match key3.get_level() {
                    Level::Low => publisher.publish_immediate(KeyEvent {key: Key::Key3, event: Event::Pressed}),
                    Level::High => publisher.publish_immediate(KeyEvent {key: Key::Key3, event: Event::Released}),
                }
            }
            3 => {
                match encoder_button.get_level() {
                    Level::Low => publisher.publish_immediate(KeyEvent {key: Key::EncoderButton, event: Event::Pressed}),
                    Level::High => publisher.publish_immediate(KeyEvent {key: Key::EncoderButton, event: Event::Released}),
                }
            }
            _ => unreachable!(),
        };
    }
}


async fn handle_encoder_interaction(mut midi_class: CustomMidi, cc: MidiCC) -> CustomMidi {
    // For encoder, send value_on, then immediately send value_off to create a momentary trigger
    // MIDI CC packet format: [CIN+Cable, Status, Controller, Value]
    // CIN for Control Change is 0xB, Cable is 0, so first byte is 0x0B
    // Status byte is 0xB0 + channel (0xBE for channel 15)
    
    let status_byte = 0xB0 | MIDI_CONFIG.channel;
    
    // Send CC with value_on
    let packet_on = [
        0x0B,           // CIN: Control Change (0xB) + Cable 0 (0x0)
        status_byte,    // Status: Control Change on configured channel
        cc.controller,  // Controller number
        cc.value_on,    // Value
    ];
    
    if let Err(e) = midi_class.write_packet(&packet_on).await {
        log::error!("Failed to send MIDI CC on: {:?}", e);
    }
    
    // Small delay to ensure the message is recognized
    embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
    
    // Send CC with value_off
    let packet_off = [
        0x0B,
        status_byte,
        cc.controller,
        cc.value_off,
    ];
    
    if let Err(e) = midi_class.write_packet(&packet_off).await {
        log::error!("Failed to send MIDI CC off: {:?}", e);
    }
    
    return midi_class;
}

async fn send_midi_cc(mut midi_class: CustomMidi, cc: MidiCC, event: Event) -> CustomMidi {
    // MIDI CC packet format: [CIN+Cable, Status, Controller, Value]
    let status_byte = 0xB0 | MIDI_CONFIG.channel;
    
    let value = match event {
        Event::Pressed => cc.value_on,
        Event::Released => cc.value_off,
    };
    
    let packet = [
        0x0B,           // CIN: Control Change (0xB) + Cable 0 (0x0)
        status_byte,    // Status: Control Change on configured channel
        cc.controller,  // Controller number
        value,          // Value
    ];
    
    if let Err(e) = midi_class.write_packet(&packet).await {
        log::error!("Failed to send MIDI CC: {:?}", e);
    }

    return midi_class;
}