extern crate gtk;
extern crate gio;
extern crate gdk;
extern crate gdk_pixbuf;

use gtk::prelude::*;
use gtk::{
    WindowExt, WidgetExt,
    TextViewExt, TextBufferExt,
};

use gio::{
    SimpleActionExt, ActionMapExt
};

use std::path::Path;
use std::ops::Deref;
use std::cell::RefCell;
use std::rc::Rc;
use page::{Page, Pages, PageExtend, PagesExtend};


pub struct WindowCore {
    win: gtk::ApplicationWindow,
    notebook: gtk::Notebook,
    pages: Pages,
    active_page: Option<Page>,
}


pub type Window = Rc<RefCell<WindowCore>>;

pub trait WindowExtend {
    fn create(app: &gtk::Application, wins: Windows) -> Window;
    fn init_actions(&self, wins: Windows);
    fn register(&self, wins: Windows);
    fn create_new_page(&self, wins: Windows) -> Page;
    fn present(&self, page: Page);
    fn init(&self);
    fn win(&self) -> gtk::ApplicationWindow;
    fn pages(&self) -> Pages;
    fn page_num(&self, page: Page) -> Option<u32>;
    fn notebook(&self) -> gtk::Notebook;
    fn get_page(&self, file: &gio::File) -> Option<Page>;
    fn open(&self, file: &gio::File, wins: Windows, warning: bool);
    fn close(&self);
    fn get_empty_page(&self) -> Option<Page>;
    fn get_active_page(&self) -> Option<Page>;
    fn set_active_page(&self, page: Option<Page>);
    fn show_about(&self);
}

impl WindowExtend for Window {
    fn create(app: &gtk::Application, wins: Windows) -> Window {
        let builder = gtk::Builder::new_from_file(Path::new("/usr/share/vanilla_text/ui/window.ui"));
        let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
        window.set_application(Some(app));

        let notebook: gtk::Notebook = builder.get_object("notebook").unwrap();

        let pages = Pages::create();


        let win = Rc::new(RefCell::new(
                WindowCore {
                    win: window.clone(),
                    notebook: notebook.clone(),
                    pages: pages.clone(),
                    active_page: None
                }));

        win.create_new_page(wins.clone());

        win.init_actions(wins.clone());

        {
            let pages = pages.clone();
            let window = window.clone();
            let win = win.clone();
            notebook.connect_page_removed(move |_, _, n| {
                pages.remove(n as usize);
                let len = pages.len();
                if len > 0 {
                    let i;
                    if n as usize == len { i = n - 1; }
                    else { i = n; }

                    let p = pages.borrow()[i as usize].clone();
                    if let Some(s) = p.tab_label().get_text() {
                        window.set_title(&s);
                    }
                } else {
                    win.close();
                }
            });
        }

        {
            let pages = pages.clone();
            let window = window.clone();
            notebook.connect_switch_page(move |_, _, n| {
                if let Some(s) = pages.borrow()[n as usize].tab_label().get_text() {
                    window.set_title(&s);
                }
            });
        }

        {
            let pages = pages.clone();
            let wins = wins.clone();
            let win = win.clone();
            let notebook = notebook.clone();
            window.connect_delete_event(move |_, _| {
                loop {
                    let p = pages.borrow_mut().pop();
                    if let Some(p) = p {
                        if p.save_confirm(wins.clone(), win.clone()) {
                            pages.borrow_mut().push(p);
                            return Inhibit(true);
                        } else {
                            pages.borrow_mut().push(p.clone());
                            notebook.detach_tab(&p.contents().clone());
                        }
                    } else {
                        break;
                    }
                }

                Inhibit(false)
            });
        }

        win.register(wins);

        win
    }

    fn create_new_page(&self, wins: Windows) -> Page {
        let page = self.borrow().pages.create_new_page(wins, self.clone());
        let notebook = self.notebook();
        notebook.append_page(&page.contents(), Some(&page.tab()));
        page
    }

    fn init_actions(&self, wins: Windows) {
        let save_action = gio::SimpleAction::new("save", None);
        {
            let win = self.clone();
            let wins = wins.clone();
            save_action.connect_activate(move |_, _| {
                win.get_active_page().unwrap().save_file(wins.clone(), win.clone());
            });
        }

        let saveas_action = gio::SimpleAction::new("saveas", None);
        {
            let win = self.clone();
            let wins = wins.clone();
            saveas_action.connect_activate(move |_, _| {
                win.get_active_page().unwrap().save_as(wins.clone(), win.clone());
            });
        }

        let close_tab_action = gio::SimpleAction::new("close_tab", None);
        {
            let win = self.clone();
            let wins = wins.clone();
            close_tab_action.connect_activate(move |_, _| {
                let p = win.get_active_page().unwrap();
                if !p.save_confirm(wins.clone(), win.clone()) {
                    win.notebook().detach_tab(&p.contents().clone());
                }
            });
        }

        let new_tab_action = gio::SimpleAction::new("new_tab", None);
        {
            let win = self.clone();
            let wins = wins.clone();
            new_tab_action.connect_activate(move |_, _| {
                win.create_new_page(wins.clone());
            });
        }

        let selectall_action = gio::SimpleAction::new("selectall", None);
        {
            let win = self.clone();
            selectall_action.connect_activate(move |_, _| {
                let p = win.get_active_page().unwrap();
                let buf = p.text_view().get_buffer().unwrap();
                let (start, end) = buf.get_bounds();
                buf.select_range(&start, &end);
            });
        }
        
        let copy_action = gio::SimpleAction::new("copy", None);
        {
            let win = self.clone();
            copy_action.connect_activate(move |_, _| {
                let p = win.get_active_page().unwrap();
                let text_view = &p.text_view();
                let clipboard = text_view.get_clipboard(&gdk::SELECTION_CLIPBOARD);
                text_view.get_buffer().unwrap().copy_clipboard(&clipboard);
            });
        }

        let paste_action = gio::SimpleAction::new("paste", None);
        {
            let win = self.clone();
            paste_action.connect_activate(move |_, _| {
                let clipboard;
                let buf;
                let editable;
                let p = win.get_active_page().unwrap();
                {
                    let text_view = &p.text_view();
                    clipboard = text_view.get_clipboard(&gdk::SELECTION_CLIPBOARD);
                    buf = text_view.get_buffer().unwrap();
                    editable = text_view.get_editable();
                }
                buf.paste_clipboard(&clipboard, None, editable);
            });
        }

        let cut_action = gio::SimpleAction::new("cut", None);
        {
            let win = self.clone();
            cut_action.connect_activate(move |_, _| {
                let clipboard;
                let buf;
                let editable;
                let p = win.get_active_page().unwrap();
                {
                    let text_view = &p.text_view();
                    clipboard = text_view.get_clipboard(&gdk::SELECTION_CLIPBOARD);
                    buf = text_view.get_buffer().unwrap();
                    editable = text_view.get_editable();
                }
                buf.cut_clipboard(&clipboard, editable);

                let text_view = &p.text_view();
                text_view.scroll_mark_onscreen(&buf.get_insert().unwrap());
            });
        }

        let open_action = gio::SimpleAction::new("open", None);
        {
            use win::run_file_chooser_dialog;
            let wins = wins.clone();
            let win = self.clone();
            open_action.connect_activate(move |_, _| {
                if let Some(file) = run_file_chooser_dialog() {
                    if let Some(p) = win.get_page(&file) {
                        win.present(p);
                    } else if wins.get_page(&file).is_some() {
                        win.open(&file, wins.clone(), true);
                    } else {
                        win.open(&file, wins.clone(), false);
                    }
                }
            });
        }

        let about_action = gio::SimpleAction::new("about", None);
        {
            let win = self.clone();
            about_action.connect_activate(move |_, _| {
                win.show_about();
            });
        }

        let w = &self.borrow().win;
        w.add_action(&save_action);
        w.add_action(&saveas_action);
        w.add_action(&close_tab_action);
        w.add_action(&new_tab_action);
        w.add_action(&selectall_action);
        w.add_action(&copy_action);
        w.add_action(&paste_action);
        w.add_action(&cut_action);
        w.add_action(&open_action);
        w.add_action(&about_action);
    }


    fn register(&self, wins: Windows) {
        wins.borrow_mut().push(self.clone());
    }

    fn get_page(&self, file: &gio::File) -> Option<Page> {
        if let Some(p) = self.pages().get_page(file) {
            return Some(p.clone());
        }

        return None;
    }


    fn present(&self, page: Page) {
        let notebook = self.notebook();
        let n = notebook.page_num(&page.contents());
        notebook.set_current_page(n);
        self.win().present();
    }

    fn init(&self) {
        self.borrow().win.show_all();
    }


    fn win(&self) -> gtk::ApplicationWindow {
        self.borrow().win.clone()
    }

    fn pages(&self) -> Pages {
        self.borrow().pages.clone()
    }

    fn notebook(&self) -> gtk::Notebook {
        self.borrow().notebook.clone()
    }

    fn page_num(&self, page: Page) -> Option<u32> {
        self.borrow().notebook.page_num(&page.contents())
    }

    fn open(&self, file: &gio::File, wins: Windows, warning: bool) {
        let page;
        if let Some(p) = self.get_empty_page() {
            page = p;
        } else {
            page = self.create_new_page(wins);
        }

        page.load_file(file);
        if warning { page.show_warning(); }
        let n = self.notebook().page_num(&page.contents());
        self.notebook().set_current_page(n);
        self.win().set_title(&page.tab_label().get_text().unwrap());
    }

    fn close(&self) {
        self.borrow().win.destroy();
    }

    fn get_empty_page(&self) -> Option<Page> {
        let pages = self.pages();
        for p in pages.borrow().deref() {
            if p.is_empty() {
                return Some(p.clone());
            }
        }

        return None;
    }

    fn get_active_page(&self) -> Option<Page> {
        if let Some(i) = self.notebook().get_current_page() {
            let pages = self.pages();
            let ref page = pages.borrow()[i as usize];
            return Some(page.clone())
        }
        None
    }

    fn set_active_page(&self, page: Option<Page>) {
        self.borrow_mut().active_page = page;
    }

    fn show_about(&self) {
        let dialog = gtk::AboutDialog::new();

        dialog.set_transient_for(&self.win());
        dialog.set_program_name("Vanilla Text");

        let logo = gdk_pixbuf::Pixbuf::new_from_file("/usr/share/icons/hicolor/128x128/apps/vanilla_text.png");
        if let Ok(logo) = logo {
            dialog.set_logo(Some(&logo));
        }

        dialog.run();
        dialog.destroy();
    }
}

pub type Windows = Rc<RefCell<Vec<Window>>>;

pub trait WindowsExtend {
    fn get_page(&self, file: &gio::File) -> Option<Page>;
    fn destroy(&self, win: Window);
    fn get_active_window(&self, app: &gtk::Application) -> Option<Window>;
}

impl WindowsExtend for Windows {
    fn get_page(&self, file: &gio::File) -> Option<Page> {
        for w in self.borrow().deref() {
            if let Some(p) = w.pages().get_page(file) {
                return Some(p.clone());
            }
        }

        return None;
    }

    fn destroy(&self, win: Window) {
        let win = win.clone();
        let i = self.borrow().iter().position(move |w| {
            w.borrow().win == win.borrow().win
        }).unwrap();

        {
            let v = self.borrow();
            v[i].borrow().win.destroy();
        }
        self.borrow_mut().remove(i);
    }

    fn get_active_window(&self, app: &gtk::Application) -> Option<Window> {
        let win = app.get_active_window().unwrap();
        for w in self.borrow().deref() {
            if w.win() == win {
                return Some(w.clone());
            }
        }

        return None;
    }
}

fn run_file_chooser_dialog() -> Option<gio::File> {
    let dialog = gtk::FileChooserDialog::new::<gtk::Window>(Some("Open File"),
                                     None,
                                     gtk::FileChooserAction::Open);
    dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
    dialog.add_button("Open", gtk::ResponseType::Accept.into());

    let file;
    if dialog.run() == gtk::ResponseType::Accept.into() {
        if let Some(path) = dialog.get_filename() {
            file = Some(gio::File::new_for_path(path.as_path()))
        } else {
            file = None
        }
    } else {
        file = None
    }

    dialog.destroy();

    file
}

