use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use std::env::{self, args};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
 
use gio::prelude::*;
use gtk::prelude::*;
extern crate glib;

use ears::{Sound, AudioController};

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
    
    //~ snd1.play();
    //~ snd2.play();
    
    //~ while snd1.is_playing() || snd2.is_playing() {}
}

fn build_ui(application: &gtk::Application) {
    
    //~ let mut snd2 = Sound::new("/usr/share/sounds/alsa/Front_Right.wav").unwrap();
    
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("drum machine");
    //~ window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    //~ window.set_keep_above(true);
    
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    //~ for i in buttons {
        //~ let button = gtk::Button::new_with_label(&i.name);
        //~ gtk::WidgetExt::set_name(&button, "buttons");
        
        //~ button.connect_clicked(move |_| {
            //~ let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
            //~ clipboard.set_text(&i.cmd);
        //~ });
        
        //~ hbox.pack_start(&button, true, true, 0);
    //~ }
    
    let snd1: Rc<RefCell<Sound>> = Rc::new(RefCell::new(Sound::new("/home/ekloeckner/.cargo/registry/src/github.com-1ecc6299db9ec823/ggez-0.5.1/resources/pew.wav").unwrap()));
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
    
    //~ let (tx, rx) = mpsc::channel();
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let child = thread::spawn(move || {
        for i in 0..20 {
            thread::sleep(Duration::from_millis(200));
            tx.send("thread out").unwrap();
        }
    });

    rx.attach(None, move |_| {
        //~ println!("{}", &text);
        snd1_clone2.borrow_mut().play();
        glib::Continue(true)
    });

    window.add(&hbox);
    window.show_all();
}
