extern crate gtk;
extern crate gio;

use gtk::{
    ResponseType, LabelExt, TextViewExt,
    TextBufferExt, NotebookExt, ButtonExt, 
    WidgetExt, FileChooserExt, DialogExt,
    ContainerExt, InfoBarExt,
    RevealerExt, BinExt, Cast
};

use gio::{
    FileExt
};

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;
use std::path::Path;

use win::{
    Window, Windows, WindowExtend, WindowsExtend
};

pub struct PageCore {
    tab: gtk::Box,
    tab_label: gtk::Label,
    revealer: gtk::Revealer,
    contents: gtk::Box,
    text_view: gtk::TextView,
    close_button: gtk::Button,
    file: Option<gio::File>,
    changed: bool,
}

impl PageCore {
    pub fn new() -> PageCore {
        let builder = gtk::Builder::new_from_file(Path::new("/usr/share/vanilla_text/ui/page.ui"));
        let tab: gtk::Box = builder.get_object("tab").unwrap();
        let label: gtk::Label = builder.get_object("label").unwrap();
        let close_button: gtk::Button = builder.get_object("close_button").unwrap();

        let contents: gtk::Box = builder.get_object("contents").unwrap();

        let revealer: gtk::Revealer = builder.get_object("revealer").unwrap();
        let info_bar = revealer.get_child().unwrap().downcast::<gtk::InfoBar>().ok().unwrap();

        {
            let revealer = revealer.clone();
            info_bar.connect_response(move |_, sig| {
                if sig == gtk::ResponseType::Close.into() {
                    revealer.set_reveal_child(false);
                }
            });
        }

        let scr_win = contents.get_children()[1].clone().downcast::<gtk::ScrolledWindow>().ok().unwrap();
        let txt_view = scr_win.get_child().unwrap().downcast::<gtk::TextView>().ok().unwrap();

        PageCore {
            tab: tab,
            tab_label: label,
            revealer: revealer,
            contents: contents,
            text_view: txt_view,
            close_button: close_button,
            file: None,
            changed: false,
        }
    }

}


pub type Page = Rc<RefCell<PageCore>>;

pub trait PageExtend {
    fn create(wins: Windows, win: Window) -> Page;
    fn file(&self) -> Option<gio::File>;
    fn set_file(&self, file: Option<gio::File>);
    fn contents(&self) -> gtk::Box;
    fn tab(&self) -> gtk::Box;
    fn tab_label(&self) -> gtk::Label;
    fn text_view(&self) -> gtk::TextView;
    fn close_button(&self) -> gtk::Button;
    fn changed(&self) -> bool;
    fn set_changed(&self, changed: bool);
    fn is_empty(&self) -> bool;
    fn load_file(&self, file: &gio::File);
    fn save_confirm(&self, wins: Windows, win: Window) -> bool;
    fn save_file(&self, wins: Windows, win: Window) -> bool;
    fn save_buffer(&self, win: Window);
    fn save_as(&self, wins: Windows, win: Window) -> bool;
    fn save_file_chooser_run(&self, win: Window) -> Option<gio::File>;
    fn show_warning(&self);
}

impl PageExtend for Page {
    fn create(wins: Windows, win: Window) -> Page {
        let page = Rc::new(RefCell::new(PageCore::new()));

        {
            let p = page.clone();
            let wins = wins.clone();
            let win = win.clone();
            page.close_button().connect_clicked(move |_| {
                if !p.save_confirm(wins.clone(),win.clone()) {
                    win.notebook().detach_tab(&p.contents().clone());
                }
            });
        }

        {
            let p = page.clone();
            page.text_view().get_buffer().unwrap().connect_changed(move |_| {
                if p.changed() {
                    return;
                }
                p.borrow_mut().changed = true;
            });
        }

        page
    }

    fn file(&self) -> Option<gio::File> {
        self.borrow().file.clone()
    }

    fn set_file(&self, file: Option<gio::File>) {
        self.borrow_mut().file = file;
    }

    fn contents(&self) -> gtk::Box {
        self.borrow().contents.clone()
    }

    fn tab(&self) -> gtk::Box {
        self.borrow().tab.clone()
    }

    fn tab_label(&self) -> gtk::Label {
        self.borrow().tab_label.clone()
    }

    fn text_view(&self) -> gtk::TextView {
        self.borrow().text_view.clone()
    }

    fn close_button(&self) -> gtk::Button {
        self.borrow().close_button.clone()
    }

    fn changed(&self) -> bool {
        self.borrow().changed
    }

    fn set_changed(&self, changed: bool) {
        self.borrow_mut().changed = changed;
    }

    fn is_empty(&self) -> bool {
        if self.file().is_some() {
            return false;
        }

        let (start, end) = self.text_view().get_buffer().unwrap().get_bounds();

        start == end
    }

    fn load_file(&self, file: &gio::File) {
        if let Ok((v, _)) = file.load_contents(None) {
            let text = String::from_utf8(v).unwrap();

            let buf = self.borrow().text_view.get_buffer().unwrap();
            buf.set_text(&text);
            self.tab_label().set_text(file.get_basename().unwrap().to_str().unwrap());
            self.set_file(Some(file.clone()));
            self.set_changed(false);
        }

    }

    fn save_confirm(&self, wins: Windows, win: Window) -> bool {
        if !self.changed() {
            return false;
        }

        let dialog = gtk::MessageDialog::new(Some(&win.win()),
                                             gtk::DIALOG_MODAL,
                                             gtk::MessageType::Warning,
                                             gtk::ButtonsType::None,
                                             "Save change before closing?");
        dialog.add_button("Close without saving", ResponseType::Reject.into());
        dialog.add_button("Cancel", ResponseType::Cancel.into());
        dialog.add_button("Save", ResponseType::Accept.into());

        let r = dialog.run();
        dialog.destroy();

        if r == ResponseType::Accept.into() {
            if self.file().is_some() {
                self.save_file(wins.clone(), win)
            } else {
                self.save_as(wins.clone(), win)
            }
        } else if r == ResponseType::Cancel.into() {
            true
        } else if r == ResponseType::Reject.into() {
            false
        } else {
            true
        }
    }

    fn save_file(&self, wins: Windows, win: Window) -> bool {
        if self.file().is_some() {
            self.save_buffer(win.clone());
            return false
        } else {
            self.save_as(wins.clone(), win.clone())
        }
    }

    fn save_buffer(&self, win: Window) {
        if let Some(buf) = self.text_view().get_buffer() {
            let (start, end) = buf.get_bounds();
            if let Some(text) = buf.get_text(&start, &end, true) {
                if self.file().as_ref().unwrap().replace_contents(text.as_bytes(),
                                         None,
                                         true,
                                         gio::FILE_CREATE_NONE,
                                         None).is_ok() {
                    self.set_changed(false);
                    self.tab_label().set_text(self.file().as_ref().unwrap().get_basename().unwrap().to_str().unwrap());
                } else {
                    let dialog = gtk::MessageDialog::new(Some(&win.win()),
                                                         gtk::DIALOG_MODAL,
                                                         gtk::MessageType::Error,
                                                         gtk::ButtonsType::Close,
                                                         "Error: Cannot save file");
                    dialog.run();
                    dialog.destroy();
                }
            }
        }
    }

    fn save_as(&self, wins: Windows, win: Window) -> bool {
        if let Some(file) = self.save_file_chooser_run(win.clone()) {
            if let Some(p) = wins.get_page(&file) {
                if p.contents() != self.contents() {
                    let dialog = gtk::MessageDialog::new(Some(&win.win()),
                                                         gtk::DIALOG_MODAL,
                                                         gtk::MessageType::Warning,
                                                         gtk::ButtonsType::Close,
                                                         "Error: Cannot save file because another window has been editing the file");
                    dialog.run();
                    dialog.destroy();
                    
                    return true;
                }
            }

            self.set_file(Some(file));
            self.save_buffer(win);

            return false;
        }

        true
    }

    fn save_file_chooser_run(&self, win: Window) -> Option<gio::File> {
        let dialog = gtk::FileChooserDialog::new::<gtk::Window>(Some("Save File"),
                                                                Some(&win.win().upcast()),
                                                                gtk::FileChooserAction::Save);
        dialog.add_button("Cancel", ResponseType::Cancel.into());
        dialog.add_button("Save", ResponseType::Accept.into());
        dialog.set_do_overwrite_confirmation(true);

        if let Some(ref f) = self.file() {
            dialog.set_filename(f.get_path().unwrap().as_path());
        } else {
            dialog.set_current_name(Path::new("untitled"));
        }

        let file;
        if dialog.run() == ResponseType::Accept.into() {
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

    fn show_warning(&self) {
        self.borrow().revealer.set_reveal_child(true);
    }
}


pub type Pages = Rc<RefCell<Vec<Page>>>;

pub trait PagesExtend {
    fn create() -> Pages;
    fn create_new_page(&self, wins: Windows, win: Window) -> Page;
    fn len(&self) -> usize;
    fn append(&self, page: Page);
    fn get_page(&self, file: &gio::File) -> Option<Page>;
    fn remove(&self, i: usize);
}

impl PagesExtend for Pages {
    fn create() -> Pages {
        Rc::new(RefCell::new(Vec::<Page>::new()))
    }

    fn create_new_page(&self, wins: Windows, win: Window) -> Page {
        let page = Page::create(wins.clone(), win);
        self.append(page.clone());

        page
    }

    fn len(&self) -> usize {
        self.borrow().len()
    }

    fn append(&self, page: Page) {
        self.borrow_mut().push(page);
    }

    fn get_page(&self, file: &gio::File) -> Option<Page> {
        for p in self.borrow().deref() {
            if let Some(ref f) = p.file() {
                if f.equal(file) {
                    return Some(p.clone());
                }
            }
        }
        
        None
    }

    fn remove(&self, i: usize) {
        self.borrow_mut().remove(i);
    }
}

