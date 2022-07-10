use crate::consts::SAMPLE_RATE;
use crate::effects;
use crate::instruments;
use crate::time;
use crate::tracks;
use crate::utils;
use crate::wave;
use midly;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use wav::{self, WAV_FORMAT_PCM};

pub fn save_m_i16_wav(wave: wave::Mono, path: &Path) -> std::io::Result<()> {
    let header = wav::Header::new(WAV_FORMAT_PCM, 1, SAMPLE_RATE, 16);
    let track = wav::BitDepth::Sixteen(
        wave.get_vec()
            .into_iter()
            .map(|x| (x * (i16::MAX as f64) / 4.0) as i16)
            .collect(),
    );
    let mut out_file = File::create(path).expect("Error while making file!");
    wav::write(header, &track, &mut out_file)
}

pub fn easy_save<W: wave::Wave>(track: W, path: &Path) {
    todo!()
}

pub fn read_midi_file<W: 'static + wave::Wave>(
    path: &Path,
) -> Result<crate::Song<W>, Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    let name = utils::user_input("What's the name of the song?");
    let mut song = crate::Song::new(name);
    let smf = midly::Smf::parse(&bytes)?;
    let header = smf.header;
    let time_keeper = Rc::new(parse_smf_for_time(&smf));
    match header.timing {
        // midly::Timing::Metrical(value) => time_keeper.set_ticks_per_beat(value.as_int()),
        midly::Timing::Timecode(_, _) => todo!(),
        midly::Timing::Metrical(_) => (),
    }
    let tracks = smf.tracks;

    for track in tracks {
        song.add_midi_track(parse_midi_track::<W>(track, Rc::clone(&time_keeper)));
    }

    Ok(song)
}

fn parse_smf_for_time(smf: &midly::Smf) -> time::TimeKeeper {
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(val) => val.as_int(),
        midly::Timing::Timecode(_, _) => todo!(),
    };
    let mut bpm = 120.0;
    let mut beats_per_bar = 4;
    let mut beat_value = 4;
    for track in &smf.tracks {
        let mut current_time = 0;
        for event in track {
            current_time += event.delta.as_int();
            use midly::MetaMessage::*;
            if let midly::TrackEventKind::Meta(msg) = event.kind {
                match msg {
                    Tempo(mspb) => bpm = 1_000_000.0 / (mspb.as_int() as f64),
                    SmpteOffset(_) => (),
                    TimeSignature(num, denom, clock_click, perquater) => {
                        beats_per_bar = num as u16;
                        beat_value = denom as u16;
                        println!("{}, {}, {}, {}", num, denom, clock_click, perquater)
                    }
                    _ => (),
                }
            }
        }
    }

    time::TimeKeeper {
        ticks_per_beat,
        beats_per_bar,
        beat_value,
        bps: bpm,
    }
}

fn parse_midi_track<W: 'static + wave::Wave>(
    track: Vec<midly::TrackEvent>,
    time_keeper: Rc<time::TimeKeeper>,
) -> tracks::MidiTrack<W> {
    let mut track_name = String::new();
    let mut track_number = 0;
    let mut current_ticks: u64 = 0;
    for event in track {
        current_ticks += event.delta.as_int() as u64;
        match event.kind {
            midly::TrackEventKind::Meta(msg) => {
                use midly::MetaMessage::*;
                match msg {
                    TrackNumber(opt) => {
                        if let Some(val) = opt {
                            track_number = val
                        }
                    }
                    TrackName(name) => {
                        track_name =
                            String::from_utf8(Vec::from(name)).expect("recieved invalid track name")
                    }
                    InstrumentName(_) => (),
                    Text(_) => todo!(),
                    EndOfTrack => (),
                    Tempo(temp) => (),
                    KeySignature(_, _) => (),
                    TimeSignature(_, _, _, _) => (),
                    MidiChannel(_) => (),
                    SmpteOffset(_) => (),
                    _ => println!("{:?}", msg),
                }
            }
            midly::TrackEventKind::Midi { channel, message } => {
                println!("{},    {:?}", channel, message)
            }
            midly::TrackEventKind::SysEx(_) => todo!(),
            midly::TrackEventKind::Escape(_) => todo!(),
        }
    }
    tracks::MidiTrack {
        name: track_name,
        instrument: Box::new(instruments::EmptyInstrument::new()),
        gain: 1.0,
        effects: effects::EffectNode::Bypass,
        notes: Vec::new(),
        time_keeper: Rc::clone(&time_keeper),
    }
}
