extern crate gtk;
extern crate gio;
use gtk::{Application, ApplicationWindow, Button, prelude::*};
use gio::prelude::*;


pub fn createDisplay() {
    let application = Application::new(
        Some("com.github.gtk-rs.examples.basic"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("GameBoy Advanced");
        window.set_default_size(960, 640);

        let button = Button::with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });
        window.add(&button);

        window.show_all();
    });

    application.run(&[]);
}