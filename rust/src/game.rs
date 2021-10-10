use gdnative::api::*;
use gdnative::prelude::*;
use fluidlite::{Settings, Synth};
use rodio::{OutputStream, OutputStreamHandle};
use rodio::buffer::SamplesBuffer;
use std::{
    env::temp_dir,
    io::Write,
    fs::File,
};

static SOUND_FONT_BYTES: &'static [u8] = include_bytes!("default.sf2");

/// The Game "class"
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signal)]
#[user_data(RwLockData<Game>)]
pub struct Game {
    _stream: OutputStream,
    synth: Synth,
    output: OutputStreamHandle,
}

unsafe impl Send for Game {}
unsafe impl Sync for Game {}

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

        if synth.sfload("default.sf2", true).is_err() {
            let mut sound_font_dir = temp_dir();
            sound_font_dir.push("default.sf2");
            if synth.sfload(&sound_font_dir, true).is_err() {
                let mut sound_font_file = File::create(&sound_font_dir).unwrap();
                sound_font_file.write_all(SOUND_FONT_BYTES).unwrap();
                synth.sfload(sound_font_dir, true).unwrap();
            }
        }

        let (_stream, output) = OutputStream::try_default().unwrap();

        Game {
            _stream,
            synth,
            output,
        }
    }

    #[export]
    unsafe fn _ready(&self, _owner: &Node) {
        OS::godot_singleton().open_midi_inputs();
    }

    // This function will be called in every frame
    #[export]
    fn _input(&self, _owner: &Node, event: Ref<InputEvent>) {
        if let Some(event) = event.cast::<InputEventMIDI>() {
            // Cast the event to TRef<T> for proper usage
            let event = unsafe { event.assume_safe() };

            // Remove all events without velocity as they are not needed
            if event.velocity() == 0 || event.channel() != 0 { return }

            // Match whether the event is a key pressed, a key released or neither
            match event.message() {
                GlobalConstants::MIDI_MESSAGE_NOTE_ON => {
                    // Prepare variables
                    let mut buffer = vec![0f32; 44100 * 2];
                    let sink = rodio::Sink::try_new(&self.output).unwrap();

                    // Digitially press the key in the synth
                    self.synth.note_on(0, event.pitch() as u32, event.velocity() as u32).unwrap();
                    self.synth.write(buffer.as_mut_slice()).unwrap();

                    // Give the sound to the sink and detach it
                    sink.append(SamplesBuffer::new(2, 44100, buffer));
                    sink.detach();

                    // Notify the keyboard that a key was pressed
                    _owner.emit_signal("note_play", &[Variant::from_i64(event.pitch())]);
                },
                GlobalConstants::MIDI_MESSAGE_NOTE_OFF => {
                    _owner.emit_signal("note_ended", &[Variant::from_i64(event.pitch())]);
                },
                _ => return
            }
        }
    }
}
