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

use ears::{Sound, AudioController};

const TRACK_COUNT: u32 = 3;
const STEP_COUNT: usize = 32;

 #[derive(Copy, Clone)]
struct Step {
    pos: usize,
    gate: bool,
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
    
    let (tx, rx): (Sender<bool>, Receiver<bool>) = channel();
    
    let button_start = gtk::ToggleButton::new_with_label("start");
    let button_start_clone = button_start.clone();
    button_start.connect_clicked(move |_| {
        tx.send(button_start_clone.get_active()).unwrap();
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
    
    let snd1: Rc<RefCell<Sound>> = Rc::new(RefCell::new(Sound::new("/usr/share/sounds/alsa/Front_Left.wav").unwrap()));
    let snd1_clone = snd1.clone();
    let snd1_clone2 = snd1.clone();
    
    let button1 = gtk::Button::new_with_label("button1");
    button1.connect_clicked(move |_| {
        snd1_clone.borrow_mut().play();
        //~ println!("sound playing");
    });
    hbox.pack_start(&button1, true, true, 0);
    
    let snd2: Rc<RefCell<Sound>> = Rc::new(RefCell::new(Sound::new("/usr/share/sounds/alsa/Front_Right.wav").unwrap()));
    let snd2_clone = snd2.clone();
    
    let button2 = gtk::Button::new_with_label("button2");
    button2.connect_clicked(move |_| {
        snd2_clone.borrow_mut().play();
        //~ println!("sound playing");
    });
    hbox.pack_start(&button2, true, true, 0);
    
    //~ snd2.play();
    
    //~ let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let child = thread::spawn(move || {
        let mut cwd = env::current_exe().unwrap();
        //~ loop {
            //~ let cur = cwd.pop();
            //~ if cur == "target" {
                //~ break;
            //~ }
        //~ }
        for _i in 0..3 { cwd.pop(); }
        cwd.push("sounds");
        //~ println!("{}", cwd.to_str().unwrap());
        
        let mut sounds = Vec::new();
        
        cwd.push("Cassette808_BD01.wav");
        sounds.push(Sound::new(cwd.to_str().unwrap()).unwrap());
        cwd.pop();
        
        cwd.push("Cassette808_Snr01.wav");
        sounds.push(Sound::new(cwd.to_str().unwrap()).unwrap());
        cwd.pop();
        
        cwd.push("Cassette808_HH_01.wav");
        sounds.push(Sound::new(cwd.to_str().unwrap()).unwrap());
        cwd.pop();
        
        //~ let mut ch01_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_BD01.wav").unwrap();
        //~ let mut ch02_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_Snr01.wav").unwrap();
        //~ let mut ch03_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_HH_01.wav").unwrap();
        
        let mut playing = false;
        
        //~ let mut ch01_steps = vec!(false; STEP_COUNT);
        //~ let mut ch02_steps = vec!(false; STEP_COUNT);
        //~ let mut ch03_steps = vec!(false; STEP_COUNT);
        
        let mut tracks = Vec::new();
        for i in 0..TRACK_COUNT {
            tracks.push(vec!(false; STEP_COUNT));
        }
        
        let mut vec = vec!(0 as usize; STEP_COUNT);
        for i in 0..STEP_COUNT {
            vec[i] = i as usize;
        }
        let mut cycle = vec.iter().cycle();
        //~ let last_frame = Instant::now();
        
        loop {
            let frame_start = Instant::now();
            //~ let dt = (now - last_frame).as_micros() as f32 / 1000000.0;
            //~ last_frame = now;
            
            match rx.try_recv() {
                Ok(val) => playing = val,
                _       => (),
                
            }
            
            for (chan, track) in channels_steps.iter().zip(tracks.iter_mut()) {
                //~ for (i, step) in chan.iter().enumerate() {
                    //~ match step.try_recv() {
                        //~ Ok(val) => {
                            //~ track[i] = val.gate;
                            //~ },
                        //~ _       => (),
                    
                    //~ }
                //~ }
                match chan.try_recv() {
                    Ok(val) => {
                        track[val.pos] = val.gate;
                        },
                    _       => (),
                
                }
                
            }
            
            if playing {
                //~ snd1_t.play();
                //~ cycle.next().unwrap();
                let step_num = *cycle.next().unwrap();
                
                /*
                if ch01_steps[step_num] {
                    ch01_snd.play();
                }
                if ch02_steps[step_num] {
                    ch02_snd.play();
                }
                if ch03_steps[step_num] {
                    ch03_snd.play();
                }
                */
                
                for (track, sound) in tracks.iter().zip(sounds.iter_mut()) {
                    if track[step_num] {
                        sound.play();
                    }
                }
            }
            
            let last_frame = Instant::now();
            
            //~ while ((Instant::now() - last_frame).as_micros() as f32) < 150000.0 {
            //~ while ((Instant::now() - frame_start).as_micros() as f32) < 100000.0 {
            while ((frame_start.elapsed()).as_micros() as f32) < 100000.0 {
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
