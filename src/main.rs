use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
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

//~ use ears::{Sound, AudioController};

use cpal::{StreamData, UnknownTypeOutputBuffer};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

const TRACK_COUNT: usize = 3;
const STEP_COUNT: usize = 32;

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
    let application =
        gtk::Application::new(Some("com.github.drum-machine"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        //~ let provider = gtk::CssProvider::new();
        //~ provider
            //~ .load_from_data(STYLE.as_bytes())
            //~ .expect("Failed to load CSS");
        //~ gtk::StyleContext::add_provider_for_screen(
            //~ &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            //~ &provider,
            //~ gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        //~ );
        
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
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
    
    for track in 0..TRACK_COUNT {
        let gui_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        //~ let mut chan = Vec::new();
        let gui_label = gtk::Label::new(Some(format!("ch{}", track + 1).as_str()));
        gui_box.pack_start(&gui_label, true, true, 0);
        let (chan_tx, chan_rx): (Sender<Step>, Receiver<Step>) = channel();
        
        for step in 0..STEP_COUNT {
            let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
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
    
    //~ let snd1: Rc<RefCell<Sound>> = Rc::new(RefCell::new(Sound::new("/usr/share/sounds/alsa/Front_Left.wav").unwrap()));
    //~ let snd1_clone = snd1.clone();
    //~ let snd1_clone2 = snd1.clone();
    
    let button1 = gtk::Button::new_with_label("button1");
    button1.connect_clicked(move |_| {
        //~ snd1_clone.borrow_mut().play();
        //~ println!("sound playing");
    });
    hbox.pack_start(&button1, true, true, 0);
    
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
    
    let (audio_thread_tx, audio_thread_rx): (Sender<[bool; TRACK_COUNT]>, Receiver<[bool; TRACK_COUNT]>) = channel();
    
    let _seq_thread = thread::spawn(move || {
        let mut playing = false;
        
        let mut tracks = Vec::new();
        for i in 0..TRACK_COUNT {
            tracks.push(vec!(false; STEP_COUNT));
        }
        
        let mut cycle_vec = vec!(0 as usize; STEP_COUNT);
        for i in 0..STEP_COUNT {
            cycle_vec[i] = i as usize;
        }
        let mut cycle = cycle_vec.iter().cycle();
        //~ let last_frame = Instant::now();
        
        loop {
            let frame_start = Instant::now();
            //~ let dt = (now - last_frame).as_micros() as f32 / 1000000.0;
            //~ last_frame = now;
            
            match start_rx.try_recv() {
                Ok(val) => playing = val,
                _       => (),
                
            }
            
            for (chan, track) in channels_steps.iter().zip(tracks.iter_mut()) {
                match chan.try_recv() {
                    Ok(val) => {
                        track[val.pos] = val.gate;
                        },
                    _ => (),
                }
            }
            
            if playing {
                let step_num = *cycle.next().unwrap();

                let mut arr_out = [false; TRACK_COUNT];
                for (i, track) in tracks.iter().enumerate() {
                    if track[step_num] {
                        //~ audio_thread_tx.send(Gate { track: i, playing: true }).unwrap();
                        arr_out[i] = true;
                    }
                }
                audio_thread_tx.send(arr_out).unwrap();
            }
            
            let last_frame = Instant::now();
            
            //~ while ((Instant::now() - last_frame).as_micros() as f32) < 150000.0 {
            //~ while ((Instant::now() - frame_start).as_micros() as f32) < 100000.0 {
            while ((frame_start.elapsed()).as_micros() as f32) < 80000.0 {
                thread::yield_now();
                //~ thread::sleep(Duration::from_millis(1));
            }
            //~ println!("{}", (Instant::now() - last_frame).as_micros());
            //~ println!("{}", (Instant::now() - frame_start).as_micros());
            //~ let last_frame = Instant::now();
            
            //~ thread::sleep(Duration::from_millis(100));
            //~ tx.send("thread out").unwrap();
        }
    });

    let _audio_thread = thread::spawn(move || {
        let mut samples = Vec::new();
        let mut sounds = Vec::new();
        
        let mut cwd = env::current_exe().unwrap();
        for _i in 0..3 { cwd.pop(); }
        cwd.push("sounds");
        
        // --
        
        cwd.push("Cassette808_BD01-16bit.wav");
        let mut reader_1 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_1 = reader_1.into_samples::<i16>()
            .map(|x| x.unwrap() / 2)
            .collect::<Vec<_>>();
        //~ let mut sample_cycle_1 = sample_vec_1.iter().cycle();
        sounds.push(Sound { len: sample_vec_1.len(), pos: 0, playing: false });
        samples.push(sample_vec_1);
        cwd.pop();
        
        //~ let mut sound_1 = Sound { len: sample_vec_1.len(), pos: 0, playing: false };
        
        // --
        
        cwd.push("Cassette808_CP_01-16bit.wav");
        let mut reader_2 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_2 = reader_2.into_samples::<i16>()
            .map(|x| x.unwrap() / 2)
            .collect::<Vec<_>>();
        //~ let mut sample_cycle_2 = sample_vec_2.iter().cycle();
        sounds.push(Sound { len: sample_vec_2.len(), pos: 0, playing: false });
        samples.push(sample_vec_2);
        cwd.pop();
        
        //~ let mut sound_2 = Sound { len: sample_vec_2.len(), pos: 0, playing: false };
        
        // --
        
        cwd.push("Cassette808_HH_01-16bit.wav");
        let mut reader_3 = hound::WavReader::open(cwd.to_str().unwrap()).unwrap();
        let sample_vec_3 = reader_3.into_samples::<i16>()
            .map(|x| x.unwrap() / 2)
            .collect::<Vec<_>>();
        //~ let mut sample_cycle_2 = sample_vec_2.iter().cycle();
        sounds.push(Sound { len: sample_vec_3.len(), pos: 0, playing: false });
        samples.push(sample_vec_3);
        cwd.pop();
        
        //~ let mut sound_2 = Sound { len: sample_vec_2.len(), pos: 0, playing: false };
        
        // --
        
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host.default_output_device().expect("no output device available");
        let mut supported_formats_range = device.supported_output_formats()
            .expect("error while querying formats");
        
        //~ for i in supported_formats_range {
            //~ println!("{:?}", i);
        //~ }
        
        //~ let format = supported_formats_range.next()
            //~ .expect("no supported format?!")
            //~ .with_max_sample_rate();
            
        let format = cpal::Format{ channels: 1, sample_rate: cpal::SampleRate(44100), data_type: cpal::SampleFormat::I16 };
        
        //~ println!("{:?}", device.name().unwrap());
        //~ println!("{:?}", format);
            
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        
        let mut sample_play = false;
        
        event_loop.run(move |stream_id, stream_result| {
            //~ let mut sample_cycle_1 = sample_vec_1.iter().cycle();
            //~ let mut sample_cycle_2 = sample_vec_2.iter().cycle();
            
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                    return;
                }
                _ => return,
            };

            match stream_data {
                //~ StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
                    //~ for elem in buffer.iter_mut() {
                        //~ *elem = u16::max_value() / 2;
                    //~ }
                //~ },
                StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
                    for elem in buffer.iter_mut() {
                        let mut mix: i16 = 0;
                        //~ if sound_1.playing || sound_2.playing {
                            //~ let mut mix: i16 = 0;
                            //~ *elem = *sample_cycle_1.next().unwrap() + *sample_cycle_2.next().unwrap();
                            
                            //~ if sound_1.playing {
                                //~ mix += sample_vec_1[sound_1.tick()];
                            //~ }
                            //~ if sound_2.playing {
                                //~ mix += sample_vec_2[sound_2.tick()];
                            //~ }
                            
                            //~ *elem = mix;
                        //~ } else {
                            //~ *elem = 0;
                        //~ }
                        for (sound, sample) in sounds.iter_mut().zip(samples.iter()) {
                            if sound.playing {
                                mix += sample[sound.tick()];
                            }
                        }
                        
                        *elem = mix;
                    }
                },
                //~ StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    //~ for elem in buffer.iter_mut() {
                        //~ *elem = 0.0;
                    //~ }
                //~ },
                _ => (),
            }
            
            match audio_thread_rx.try_recv() {
                Ok(v) => {
                    //~ sounds[val.track].play();
                    for (i, val) in v.iter().enumerate() {
                        if *val {
                           sounds[i].play();
                        }
                    }
                },
                _ => (),
                
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
