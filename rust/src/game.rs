use gdnative::api::*;
use gdnative::prelude::*;
use fluidlite::{Settings, Synth};
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use rodio::{OutputStream, OutputStreamHandle};

/// The Game "class"
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signal)]
#[user_data(gdnative::nativescript::user_data::MutexData<Game>)]
pub struct Game {
    internal: Arc<Mutex<Internal>>
}

struct Internal {
    _stream: OutputStream,
    synth: Synth,
    output: OutputStreamHandle,
}

unsafe impl Send for Game {}
unsafe impl Send for Internal {}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_signal(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "note_play", 
            args: &[SignalArgument {
                name: "note",
                default: Variant::from_i64(-1),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }], 
        });
        builder.add_signal(Signal {
            name: "note_ended",
            args: &[SignalArgument {
                name: "note",
                default: Variant::from_i64(-1),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        godot_print!("Game is created!");
        let synth = Synth::new(Settings::new().unwrap()).unwrap();
        synth.sfload("default.sf2", true).unwrap();
        let (_stream, output) = OutputStream::try_default().unwrap();

        let internal = Internal {
            _stream,
            synth,
            output,
        };

        Game {
            internal: Arc::new(Mutex::new(internal))
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {
        OS::godot_singleton().open_midi_inputs();
    }

    // This function will be called in every frame
    #[export]
    fn _input(&self, _owner: &Node, event: Ref<InputEvent>) {
        let mut buffer = vec![0f32; 441000];
        let event = unsafe { event.assume_safe() };
        let mut note_number = -1;

        let internal_arc = Arc::clone(&self.internal);
        let internal = internal_arc.lock().unwrap();

        // Determine the type of event
        let mut note_pressed = event.is_pressed();

        // Get the data from the event by casting it into the correct format
        if let Some(event) = event.cast::<InputEventMIDI>() {
            note_number = event.pitch();
            note_pressed = event.message() == 9;
        }
        else if let Some(event) = event.cast::<InputEventKey>() {
            note_number = event.scancode();
        }

        // Return if the note_number doesn't meet the requirements
        if i8::try_from(note_number).is_err() || note_number.is_negative() { return }


        godot_print!("Note {} has been pressed: {}", note_number, note_pressed);
        
        if note_pressed {
            internal.synth.note_on(0, note_number as u32, 127).unwrap();
            _owner.emit_signal(
                "note_play",
                &[Variant::from_i64(note_number)]
            );
        }
        else {
            _owner.emit_signal(
                "note_ended",
                &[Variant::from_i64(note_number)]
            );
            if internal.synth.note_off(0, note_number as u32).is_err() { return }
        }

        internal.synth.write(buffer.as_mut_slice()).unwrap();
        internal.output.play_raw(
            rodio::buffer::SamplesBuffer::new(1, 441000, buffer)
        ).unwrap();
    }
}
