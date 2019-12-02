use std::fs;
use std::io::{self, BufRead};
use std::path;
use std::process;
use std::env::{self, args};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
 
use gio::prelude::*;
use gtk::prelude::*;
extern crate glib;

use cpal::{StreamData, UnknownTypeOutputBuffer};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

const PROJECT_NAME: &str = "drum-machine";
const TRACK_COUNT: usize = 6;
const STEP_COUNT: usize = 32;

#[derive(Copy, Clone)]
struct Sequencer {
    pos:              usize,
    steps:            usize,
    updated:          bool,
    sample:           i16,
    samplerate:       u32,
    samples_per_step: f32,
    samples_carry:    f32,
}

impl Sequencer {
    fn new(tempo: f32, samplerate: u32, steps: usize) -> Sequencer {
        let secs_per_step: f32 = 60.0 / tempo / 8.0;
        let samples_per_step = secs_per_step * samplerate as f32;
        Sequencer {
            pos:              0,
            steps:            steps,
            updated:          true,
            sample:           0,
            samplerate:       samplerate,
            samples_per_step: samples_per_step,
            samples_carry:    0.0,
        }
    }
    
    fn tick(&mut self) {
        if self.sample < (self.samples_per_step.trunc() - 1.0) as i16 {
            self.sample += 1;
            //~ println!("carry: {}", self.samples_carry);
        } else {
            if self.samples_carry < 1.0 {
                //~ print!("sample: {} | ", self.sample);
                //~ print!("samples/step: {} | ", self.samples_per_step);
                self.sample = 0;
                self.samples_carry += self.samples_per_step - self.samples_per_step.trunc();
                if self.pos < self.steps - 1 {
                    self.pos += 1;
                    self.updated = true;
                } else {
                    self.pos = 0;
                    self.updated = true;
                }
                //~ println!("step {}", self.pos);
            } else {
                self.sample += 1;
                self.samples_carry -= 1.0;
                //~ println!("carry: {}", self.samples_carry);
            }
        }
    }
    
    fn set_tempo(&mut self, tempo: f32) {
        let secs_per_step: f32 = 60.0 / tempo / 8.0;
        self.samples_per_step = secs_per_step * self.samplerate as f32;
        self.samples_carry = 0.0;
    }
}

#[derive(Copy, Clone)]
struct Step {
    pos: usize,
    gate: bool,
}

#[derive(Copy, Clone)]
struct Sound {
    len:     usize,
    pos:     usize,
    playing: bool,
}

impl Sound {
    fn play(&mut self) {
        if !self.playing {
            self.pos = 0;
            self.playing = true;
        } else {
            self.pos = 0;
        }
    }
    
    fn tick(&mut self) -> usize {
        let mut output: usize = 0;
        if self.pos < self.len {
            output = self.pos;
            self.pos += 1;
        } else {
            self.playing = false;
        }
        output
    }
}

fn main() {
    //~ let mut css_path = env::current_exe().unwrap();
    //~ for _i in 0..3 { css_path.pop(); }
    //~ css_path.push("css");
    let mut css_path = find_dir("css", PROJECT_NAME);
    css_path.push("main.css");
    let style = fs::read_to_string(css_path.to_str().unwrap()).unwrap();
    
    let application =
        gtk::Application::new(Some("com.github.drum-machine"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(move |app| {
        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(style.as_str().as_bytes())
            .expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn find_dir(dir_name: &str, project_name: &str) -> path::PathBuf {
    let mut path_out = env::current_exe().unwrap();
    let mut res = path_out.pop();
    loop {
        if res {
            if path_out.as_path().file_name().unwrap().to_str().unwrap() == project_name {
                path_out.push(dir_name);
                break;
            }
            res = path_out.pop();
        } else {
            break;
        }
    }
    path_out
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("drum machine");
    window.set_position(gtk::WindowPosition::Center);
    
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    
    let (start_tx, start_rx): (Sender<bool>, Receiver<bool>) = channel();
    
    let button_start = gtk::ToggleButton::new_with_label("start");
    let button_start_clone = button_start.clone();
    button_start.connect_clicked(move |_| {
        start_tx.send(button_start_clone.get_active()).unwrap();
    });
    hbox.pack_start(&button_start, true, true, 0);
    
    // --
    
    let mut channels_steps = Vec::new();
    
    let labels =   ["bassdrum",
                    "snare",
                    "clap",
                    "cowbell",
                    "closed HH",
                    "open HH",
    ];
    
    for track in 0..TRACK_COUNT {
        let gui_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        //~ let mut chan = Vec::new();
        let gui_label = gtk::Label::new(Some(labels[track]));
        gtk::WidgetExt::set_name(&gui_label, "track-label");
        gui_box.pack_start(&gui_label, true, true, 0);
        let (chan_tx, chan_rx): (Sender<Step>, Receiver<Step>) = channel();
        
        for step in 0..STEP_COUNT {
            let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
            if (step / 8) % 2 == 0 {
                gtk::WidgetExt::set_name(&btn, "step-odd");
            } else {
                gtk::WidgetExt::set_name(&btn, "step-even");
            }
            let btn_clone = btn.clone();
            //~ let (step_tx, step_rx): (Sender<Step>, Receiver<Step>) = channel();
            let step_tx = chan_tx.clone();
            btn.connect_clicked(move |_| {
                step_tx.send(Step { pos: step, gate: btn_clone.get_active() }).unwrap();
                //~ println!("{} state: {}", step, btn_clone.get_active());
            });
            //~ chan.push(step_rx);
            
            gui_box.pack_start(&btn, true, true, 0);
        }
        
        //~ channels_steps.push(chan);
        channels_steps.push(chan_rx);
        main_box.pack_start(&gui_box, true, true, 0);
    }
    
    // --
    let (tempo_tx, tempo_rx): (Sender<f32>, Receiver<f32>) = channel();
    
    let adj = gtk::Adjustment::new(120.0, 1.0, 300.0, 1.0, 1.0, 1.0);
    //~ let tempo_in = gtk::SpinButton::new_with_range(1.0, 300.0, 1.0);
    let tempo_in = gtk::SpinButton::new(Some(&adj), 10.0, 3);
    tempo_in.connect_input(move |tempo_in| {
        let text = tempo_in.get_text().unwrap();
        //~ match text.parse::<f32>() {
            
        //~ }
        
        match text.parse::<f64>() {
            Ok(value) => {
                tempo_tx.send(value as f32).unwrap();
                Some(Ok(value))
            },
            Err(_) => Some(Err(())),
        }
    });
    hbox.pack_start(&tempo_in, true, true, 0);
    
    //~ let snd2: Rc<RefCell<Sound>> = Rc::new(RefCell::new(Sound::new("/usr/share/sounds/alsa/Front_Right.wav").unwrap()));
    //~ let snd2_clone = snd2.clone();
    
    let button2 = gtk::Button::new_with_label("button2");
    button2.connect_clicked(move |_| {
        //~ snd2_clone.borrow_mut().play();
        //~ println!("sound playing");
    });
    hbox.pack_start(&button2, true, true, 0);
    
    //~ snd2.play();
    
    //~ let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let _audio_thread = thread::spawn(move || {
        let mut samples = Vec::new();
        let mut sounds = Vec::new();
        
        //~ let mut cwd = env::current_exe().unwrap();
        //~ for _i in 0..3 { cwd.pop(); }
        //~ cwd.pop();
        //~ cwd.push("sounds");
        let mut cwd = find_dir("sounds", PROJECT_NAME);
        
        let sample_scale: i16 = 3;
        
        let names =    ["Cassette808_BD01-16bit.wav",
                        "Cassette808_Snr03-16bit.wav",
                        "Cassette808_CP_01-16bit.wav",
                        "Cassette808_Cow01-16bit.wav",
                        "Cassette808_HH_01-16bit.wav",
                        "Cassette808_HHo_01-16bit.wav",
        ];
        
        for name in names.iter() {
            cwd.push(name);
            let mut wavreader = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
            let sample_vec = wavreader.into_samples::<i16>()
                .map(|x| x.unwrap() / sample_scale)
                .collect::<Vec<_>>();
            sounds.push(Sound { len: sample_vec.len(), pos: 0, playing: false });
            samples.push(sample_vec);
            cwd.pop();
        }
        
        // --
        /*
        cwd.push("Cassette808_BD01-16bit.wav");
        let mut reader_1 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_1 = reader_1.into_samples::<i16>()
            .map(|x| x.unwrap() / sample_scale)
            .collect::<Vec<_>>();
        sounds.push(Sound { len: sample_vec_1.len(), pos: 0, playing: false });
        samples.push(sample_vec_1);
        cwd.pop();
        
        // --
        
        cwd.push("Cassette808_Snr03-16bit.wav");
        let mut reader_2 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_2 = reader_2.into_samples::<i16>()
            .map(|x| x.unwrap() / sample_scale)
            .collect::<Vec<_>>();
        sounds.push(Sound { len: sample_vec_2.len(), pos: 0, playing: false });
        samples.push(sample_vec_2);
        cwd.pop();

        // --
        
        cwd.push("Cassette808_HH_01-16bit.wav");
        let mut reader_3 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_3 = reader_3.into_samples::<i16>()
            .map(|x| x.unwrap() / sample_scale)
            .collect::<Vec<_>>();
        sounds.push(Sound { len: sample_vec_3.len(), pos: 0, playing: false });
        samples.push(sample_vec_3);
        cwd.pop();

        // --
        
        cwd.push("Cassette808_HHo_01-16bit.wav");
        let mut reader_4 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_4 = reader_4.into_samples::<i16>()
            .map(|x| x.unwrap() / sample_scale)
            .collect::<Vec<_>>();
        sounds.push(Sound { len: sample_vec_4.len(), pos: 0, playing: false });
        samples.push(sample_vec_4);
        cwd.pop();
        */
        // --
        
        let mut playing = false;
        let mut tempo: f32 = 120.0;
        let mut seq = Sequencer::new(tempo, 44100, STEP_COUNT);
        let mut tracks = Vec::new();
        for _i in 0..TRACK_COUNT {
            tracks.push(vec!(false; STEP_COUNT));
        }
        
        // --
        
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host.default_output_device().expect("no output device available");
        let format = cpal::Format{ channels: 1, sample_rate: cpal::SampleRate(44100), data_type: cpal::SampleFormat::I16 };
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        
        event_loop.run(move |stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                    return;
                }
                _ => return,
            };

            match stream_data {
                StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
                    for elem in buffer.iter_mut() {
                        let mut mix: i16 = 0;
                        
                        if playing {
                            if seq.updated {
                                for (track, sound) in tracks.iter().zip(sounds.iter_mut()) {
                                    if track[seq.pos] {
                                        sound.play();
                                    }
                                }
                                seq.updated = false;
                            }
                            
                            for (sound, sample) in sounds.iter_mut().zip(samples.iter()) {
                                if sound.playing {
                                    //~ mix += sample[sound.tick()];
                                    mix = mix.saturating_add(sample[sound.tick()]);
                                }
                            }
                            seq.tick();
                        }
                        
                        *elem = mix;
                    }
                },
                _ => (),
            }
            
            /*
            match audio_thread_rx.try_recv() {
                Ok(v) => {
                    for (i, val) in v.iter().enumerate() {
                        if *val {
                           sounds[i].play();
                        }
                    }
                },
                _ => (),
                
            }
            */
            
            match start_rx.try_recv() {
                Ok(val) => playing = val,
                _ => (),
                
            }
            
            match tempo_rx.try_recv() {
                Ok(val) => {
                    tempo = val;
                    seq.set_tempo(tempo);
                },
                _ => (),
                
            }
            
            for (chan, track) in channels_steps.iter().zip(tracks.iter_mut()) {
                match chan.try_recv() {
                    Ok(val) => {
                        track[val.pos] = val.gate;
                    },
                    _ => (),
                }
            }
        });
    });

    /*
    rx.attach(None, move |_| {
        //~ println!("{}", &text);
        //~ snd1_clone2.borrow_mut().play();
        glib::Continue(true)
    });
    */
    
    main_box.pack_start(&hbox, true, true, 0);
    window.add(&main_box);
    window.show_all();
}
