use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use std::env::{self, args};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;
 
use gio::prelude::*;
use gtk::prelude::*;
extern crate glib;

use ears::{Sound, AudioController};

const TRACK_COUNT: u32 = 3;

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
        let mut chan = Vec::new();
        let gui_label = gtk::Label::new(Some(format!("ch{}", track + 1).as_str()));
        gui_box.pack_start(&gui_label, true, true, 0);
        
        for step in 0..16 {
            let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
            let btn_clone = btn.clone();
            let (step_tx, step_rx): (Sender<bool>, Receiver<bool>) = channel();
            btn.connect_clicked(move |_| {
                step_tx.send(btn_clone.get_active()).unwrap();
                //~ println!("{} state: {}", step, btn_clone.get_active());
            });
            chan.push(step_rx);
            
            gui_box.pack_start(&btn, true, true, 0);
        }
        
        channels_steps.push(chan);
        main_box.pack_start(&gui_box, true, true, 0);
    }
    
    // --
    
    /*
    // --drum channel 1 GUI widgets
    let ch01_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mut ch01_chan = Vec::new();
    
    let ch01_label = gtk::Label::new(Some("ch01"));
    ch01_box.pack_start(&ch01_label, true, true, 0);
    
    for step in 0..16 {
        let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
        let btn_clone = btn.clone();
        let (step_tx, step_rx): (Sender<bool>, Receiver<bool>) = channel();
        btn.connect_clicked(move |_| {
            step_tx.send(btn_clone.get_active()).unwrap();
            //~ println!("{} state: {}", step, btn_clone.get_active());
        });
        ch01_chan.push(step_rx);
        
        ch01_box.pack_start(&btn, true, true, 0);
    }
    
    main_box.pack_start(&ch01_box, true, true, 0);
    
    // --drum channel 2 GUI widgets
    let ch02_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mut ch02_chan = Vec::new();
    
    let ch02_label = gtk::Label::new(Some("ch02"));
    ch02_box.pack_start(&ch02_label, true, true, 0);
    
    for step in 0..16 {
        let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
        let btn_clone = btn.clone();
        let (step_tx, step_rx): (Sender<bool>, Receiver<bool>) = channel();
        btn.connect_clicked(move |_| {
            step_tx.send(btn_clone.get_active()).unwrap();
            //~ println!("{} state: {}", step, btn_clone.get_active());
        });
        ch02_chan.push(step_rx);
        
        ch02_box.pack_start(&btn, true, true, 0);
    }
    
    main_box.pack_start(&ch02_box, true, true, 0);
    
    // --drum channel 3 GUI widgets
    let ch03_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mut ch03_chan = Vec::new();
    
    let ch03_label = gtk::Label::new(Some("ch03"));
    ch03_box.pack_start(&ch03_label, true, true, 0);
    
    for step in 0..16 {
        let btn = gtk::ToggleButton::new_with_label(format!("{}", step + 1).as_str());
        let btn_clone = btn.clone();
        let (step_tx, step_rx): (Sender<bool>, Receiver<bool>) = channel();
        btn.connect_clicked(move |_| {
            step_tx.send(btn_clone.get_active()).unwrap();
            //~ println!("{} state: {}", step, btn_clone.get_active());
        });
        ch03_chan.push(step_rx);
        
        ch03_box.pack_start(&btn, true, true, 0);
    }
    
    main_box.pack_start(&ch03_box, true, true, 0);
    
    */
    
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
        let mut sounds = Vec::new();
        sounds.push(Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_BD01.wav").unwrap());
        sounds.push(Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_Snr01.wav").unwrap());
        sounds.push(Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_HH_01.wav").unwrap());
        
        //~ let mut ch01_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_BD01.wav").unwrap();
        //~ let mut ch02_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_Snr01.wav").unwrap();
        //~ let mut ch03_snd = Sound::new("/home/pi/Downloads/Cassette808_Samples/Cassette808_HH_01.wav").unwrap();
        
        let mut playing = false;
        
        let mut ch01_steps = vec!(false; 16);
        let mut ch02_steps = vec!(false; 16);
        let mut ch03_steps = vec!(false; 16);
        
        let mut tracks = Vec::new();
        for i in 0..TRACK_COUNT {
            tracks.push(vec!(false; 16));
        }
        
        let mut vec = vec!(0 as usize; 16);
        for i in 0..16 {
            vec[i] = i as usize;
        }
        let mut cycle = vec.iter().cycle();
        
        loop {
            match rx.try_recv() {
                Ok(val) => playing = val,
                _       => (),
                
            }
            
            for (chan, track) in channels_steps.iter().zip(tracks.iter_mut()) {
                for (i, step) in chan.iter().enumerate() {
                    match step.try_recv() {
                        Ok(val) => {
                            //~ println!("step {} - {}", i, val);
                            track[i] = val;
                            },
                        _       => (),
                    
                    }
                }
            }
            
            /*
            for (i, step) in ch01_chan.iter().enumerate() {
                match step.try_recv() {
                    Ok(val) => {
                        //~ println!("step {} - {}", i, val);
                        ch01_steps[i] = val;
                        },
                    _       => (),
                
                }
            }
            
            for (i, step) in ch02_chan.iter().enumerate() {
                match step.try_recv() {
                    Ok(val) => {
                        //~ println!("step {} - {}", i, val);
                        ch02_steps[i] = val;
                        },
                    _       => (),
                
                }
            }
            
            for (i, step) in ch03_chan.iter().enumerate() {
                match step.try_recv() {
                    Ok(val) => {
                        //~ println!("step {} - {}", i, val);
                        ch03_steps[i] = val;
                        },
                    _       => (),
                
                }
            }
            */
            
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
            thread::sleep(Duration::from_millis(200));
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
