extern crate gtk;
extern crate gio;
extern crate gdk;
extern crate gdk_pixbuf;

mod win;
mod page;

use std::env::Args;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::{ GtkApplicationExt };

use gio::{ 
    ApplicationExt, ApplicationExtManual,
    ActionMapExt, SimpleActionExt
};

use win::{
    Window, WindowExtend, Windows, WindowsExtend
};

fn init_actions(app: &gtk::Application, wins: &Windows) {
    let new_window_action = gio::SimpleAction::new("new_window", None);
    {
        let app = app.clone();
        let wins = wins.clone();
        new_window_action.connect_activate(move |_, _| {
            let w = Window::create(&app, wins.clone());
            w.init();
        });
    }

    let quit_action = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit_action.connect_activate(move |_, _| {
            app.quit();
        });
    }

    app.add_action(&new_window_action);
    app.add_action(&quit_action);
}

fn init_accels(app: &gtk::Application) {
    app.add_accelerator("<Ctrl>q", "app.quit", None);
    app.add_accelerator("<Ctrl>n", "app.new_window", None);
    app.add_accelerator("<Ctrl>o", "win.open", None);
    app.add_accelerator("<Ctrl>s", "win.save", None);
    app.add_accelerator("<Shift><Ctrl>s", "win.saveas", None);
    app.add_accelerator("<Ctrl>w", "win.close_tab", None);
    app.add_accelerator("<Ctrl>t", "win.new_tab", None);
    app.add_accelerator("<Ctrl>a", "win.selectall", None);
    app.add_accelerator("<Ctrl>c", "win.copy", None);
    app.add_accelerator("<Ctrl>v", "win.paste", None);
    app.add_accelerator("<Ctrl>x", "win.cut", None);
}

fn run(args: Args) {
    match gtk::Application::new("com.github.koji-m.vanilla_text", gio::APPLICATION_HANDLES_OPEN) {
        Ok(app) => {
            let wins = Rc::new(RefCell::new(Vec::<Window>::new()));

            {
                let wins = wins.clone();
                app.connect_startup(move |app| {
                    init_actions(app, &wins);
                    init_accels(app);
                    let builder = gtk::Builder::new_from_file(Path::new("/usr/share/myedit/ui/menu.ui"));

                    let app_menu: gio::Menu = builder.get_object("app_menu").unwrap();
                    app.set_app_menu(&app_menu);

                    let menu_bar: gio::Menu = builder.get_object("menu_bar").unwrap();
                    app.set_menubar(&menu_bar);
                });
            }

            {
                let wins = wins.clone();
                app.connect_activate(move |app| {
                    let w = Window::create(app, wins.clone());
                    w.init();
                });
            }

            {
                let wins = wins.clone();
                app.connect_open(move |app, files, _| {
                    let w = Window::create(app, wins.clone());
                    for file in files {
                        if let Some(p) = wins.get_page(&file) {
                            w.present(p);
                        } else {
                            w.open(&file, wins.clone(), false);
                        }
                    }
                    w.init();
                });
            }


            let args: Vec<String> = args.collect();
            let argv: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

            app.run(argv.as_slice());
        },

        Err(_) => {
            println!("Application run error");
        }
    };
}


fn main() {
    run(std::env::args());
}

