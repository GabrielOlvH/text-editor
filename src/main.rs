//#![windows_subsystem = "windows"]
mod cache;
mod hooks;
mod search_hooks;

use std::arch::x86_64::_xgetbv;
use std::cell::{RefCell, RefMut};
use std::cmp::{max, min};
use std::path::Path;
use std::rc::Rc;
use slint::{CloseRequestResponse, Image, Model, SharedString, VecModel, Weak};
use log::log;
use regex::Regex;
use rfd::FileDialog;
use slint::platform::Key;
use slint::private_unstable_api::re_exports::{KeyboardModifiers, KeyEvent};
use crate::cache::Database;
use crate::hooks::{handle_click_file_tree, handle_delete, handle_new_file_button, handle_rename, handle_textbox_edit};
use crate::search_hooks::{on_move_down, on_pressed_enter, on_search};

slint::include_modules!();

static mut CURRENT_FILE: Option<String> = None;

fn main() -> Result<(), slint::PlatformError> {
    let mut db = Rc::new(RefCell::new(Database::new("cache")));
    db.borrow_mut().load().expect("Failed to load db");

    let ui = AppWindow::new()?;


    let model = Rc::new(slint::VecModel::from(vec![]));
    build_file_tree(Rc::clone(&db), model.clone());
    ui.set_files(model.clone().into());
    handle_textbox_edit(Rc::clone(&db), ui.as_weak());
    handle_click_file_tree(Rc::clone(&db), model.clone(), ui.as_weak());
    handle_new_file_button(Rc::clone(&db), model.clone(), ui.as_weak());
    handle_window_events(Rc::clone(&db), ui.as_weak());
    handle_rename(Rc::clone(&db), model.clone(), ui.as_weak());
    handle_delete(Rc::clone(&db), model.clone(), ui.as_weak());


    on_search(Rc::clone(&db), ui.as_weak());
    on_pressed_enter(Rc::clone(&db), ui.as_weak());
    on_move_down(Rc::clone(&db), ui.as_weak());

    let clone = Rc::clone(&db);

    let ui_handle = ui.as_weak();
    ui_handle.unwrap().on_close(move || {
        let ui = ui_handle.unwrap();
        let mut binding = clone.borrow_mut();
        ui.window().hide();
        println!("Window close requested");
        open_file(&mut binding, ui_handle.clone(), None);
        binding.save_all().expect("Failed to save all files on exit!");
    });

    let ui_handle = ui.as_weak();
    ui_handle.unwrap().on_process_shortcut(move |event: KeyEvent| {
        let ui = ui_handle.unwrap();
        println!("received {}",event.text);
        if event.modifiers.control && event.text.eq_ignore_ascii_case("t") {
            ui.invoke_show_search_popup();
        }
    });
    let ui_handle = ui.as_weak();
    ui_handle.unwrap().on_close_popups(move || {
        let ui = ui_handle.unwrap();
        ui.invoke_hide_popups();
    });

    let ui_handle = ui.as_weak();
    ui_handle.unwrap().on_open_background_image_selection_dialog(move || {
        let ui = ui_handle.unwrap();
        if let Some(path) = FileDialog::new().add_filter("Images",&vec!["png", "jpg"]).pick_file() {
            let p = path.display().to_string().clone();
            let path = Path::new(p.as_str());

            ui.invoke_set_background_image(Image::load_from_path(path).unwrap());
            return SharedString::from(p);
        } else {
            return SharedString::from("None");
        }

    });
    let clone = Rc::clone(&db);
    let ui_handle = ui.as_weak();
    ui_handle.unwrap().on_open_working_directory_selection_dialog(move || {
        let ui = ui_handle.unwrap();
        let mut binding = clone.borrow_mut();
        if let Some(path) = FileDialog::new().pick_folder() {
            let p = path.display().to_string().clone();
            println!("Changing directories!");
            open_file(&mut binding, ui.as_weak(), None);
            binding.change_dirs(p.clone());
            binding.load().expect("Failed to load db");
            println!("Loaded new directory!");
            drop(binding);
            build_file_tree(Rc::clone(&db), model.clone());
            ui.invoke_hide_popups();

            return SharedString::from(p.clone());
        } else {
            println!("aaaasdfadf\n");
            return SharedString::from("None");
        }

    });



    ui.run()
}




fn handle_window_events(db: Rc<RefCell<Database>>, weak: Weak<AppWindow>) {
    let ui_handle = weak.clone();
    ui_handle.unwrap().window().on_close_requested(move || {
        println!("Window close requested");
        open_file(&mut db.borrow_mut(), ui_handle.clone(), None);
        db.borrow().save_all().expect("Failed to save all files on exit!");
        return CloseRequestResponse::HideWindow;
    });
    let ui_handle = weak.clone();
    ui_handle.unwrap().on_mouse_move(move |delta_x, delta_y| {
        let pin_win_clone = ui_handle.unwrap();
        let logical_pos = pin_win_clone.window().position().to_logical(pin_win_clone.window().scale_factor());
        pin_win_clone.window().set_position(slint::LogicalPosition::new(logical_pos.x + delta_x, logical_pos.y + delta_y));
    });


    let ui_handle = weak.clone();
    ui_handle.unwrap().on_maximize(move || {
        let pin_win_clone = ui_handle.unwrap();
        pin_win_clone.window().set_maximized(!pin_win_clone.window().is_maximized());
    });
}

fn build_file_tree(db: Rc<RefCell<Database>>, model: Rc<VecModel<FileTreeItemData>>) {
    let mut vector: Vec<FileTreeItemData> = Vec::new();

    let cl = Rc::clone(&db);
    let binding = &cl.borrow_mut();

    let mut keys = binding.keys().clone();
    keys.sort();

    let mut paths_added: Vec<String> = Vec::new();
    for key in keys {
        push_files(binding, &mut vector, &mut paths_added, String::from(""), key.clone(), 0);
    }

    model.set_vec(vector);

}

fn push_files(db: &RefMut<Database>, vector: &mut Vec<FileTreeItemData>, paths_added: &mut Vec<String>, prefix: String, current: String, depth: i32) {
    let split = current.split("/").collect::<Vec<_>>();
    if split.clone().len() > 1 {

        let mut new_prefix = prefix.clone().trim().to_string();
        if prefix.is_empty() {
            new_prefix.push_str(format!("{}", split[0]).as_str());
        } else {
            new_prefix.push_str(format!("/{}", split[0]).as_str());
        }

        let len = vector.clone().len();

        let full_path = if prefix.trim().is_empty() { split[0].to_string().trim().to_string() } else { format!("{}/{}", prefix, split[0]).to_string().trim().to_string() };
        if !full_path.is_empty() && !paths_added.contains(&full_path) {
            vector.push(FileTreeItemData {
                name: SharedString::from(split[0]),
                index: len as i32,
                full_path: SharedString::from(full_path.clone()),
                ident: depth,
                open: false,
                r#type: SharedString::from("folder")
            });
         //   println!("Pushed {}", full_path);
            paths_added.push(full_path.clone());
        }

        if !db.collapsed(full_path.as_str()) {
            return push_files(db, vector, paths_added, new_prefix, split.iter().skip(1).cloned().collect::<Vec<_>>().join("/"), depth + 1)
        }
    } else {
        let len = vector.clone().len();
        let full_path = if prefix.trim().is_empty() { current.to_string().trim().to_string() } else { format!("{}/{}", prefix, current).to_string().trim().to_string() };
        if !full_path.is_empty() && !paths_added.contains(&full_path) {
            vector.push(FileTreeItemData {
                name: SharedString::from(current.clone()),
                index: len as i32,
                full_path: SharedString::from(full_path.clone()),
                ident: depth,
                open: false,
                r#type: SharedString::from("file")
            });
            //println!("Pushed {}", full_path);

            paths_added.push(full_path.clone());
        }
    }
}


fn remove_invalid_dirs(str: String) -> String {
    return if str.starts_with("/") {
        remove_invalid_dirs(str[1..].to_string())
    } else if str.find("//").is_some() {
        remove_invalid_dirs(str.to_string().replace("//", "/"))
    } else {
        str
    }
}


fn open_file(db: &mut Database, ui_handle: Weak<AppWindow>, file: Option<String>) {
    unsafe {
        let ui = ui_handle.unwrap();
        if CURRENT_FILE.is_some() {
            db.insert(CURRENT_FILE.clone().unwrap().as_str().to_owned().clone(), ui.invoke_get_current_box().as_str().to_owned().clone());
        }
        if file.is_some() {
            let f = file.unwrap();
            for x in 0..ui.get_files().row_count() {
                let mut a = ui.get_files().row_data(x).unwrap();
                a.open = ui.get_files().row_data(x).unwrap().full_path == f;
                ui.get_files().set_row_data(x, a);
            }
            ui.invoke_set_open_file(SharedString::from(f.clone()), SharedString::from(db.get_contents(f.as_str())));
            CURRENT_FILE = Some(f.clone());
        } else {
            CURRENT_FILE = None;
        }
    }
}