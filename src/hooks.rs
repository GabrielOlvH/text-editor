use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use rfd::FileDialog;
use slint::{ComponentHandle, Image, Model, SharedString, VecModel, Weak};
use slint::private_unstable_api::re_exports::KeyEvent;
use crate::{AppWindow, build_file_tree, CURRENT_FILE, FileTreeItemData, open_file, remove_invalid_dirs};
use crate::cache::Database;
use crate::state::State;


pub fn handle_textbox_edit(db: Rc<RefCell<Database>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_edited(move |_| {
        let ui = ui_handle.unwrap();
        let mut binding = db.borrow_mut();
        unsafe {
            if CURRENT_FILE.is_none() {
                ui.invoke_new_file();
            } else {
                binding.mark_dirty(CURRENT_FILE.clone().unwrap());
            }
        }
        ();
    });
}

pub fn handle_click_file_tree(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_clicked(move |index: i32| {
        let mut binding = db.borrow_mut();
        let opt = model.row_data(index.try_into().unwrap());
        if opt.is_some() {

            let item = opt.unwrap();
            println!("Clicked on {}", item.full_path);
            if binding.contains(item.full_path.as_str()) {
                open_file(&mut binding, &mut state.borrow_mut(), ui_handle.clone(), Some(item.full_path.to_string()));
            } else {
                binding.toggle_collapse(item.full_path.as_str());
                println!("Collapsed {}", item.full_path);
                drop(binding);
                build_file_tree(db.clone(), model.clone());
            }
        }
    });
}

pub fn handle_new_file_button(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_new_file(move || {
        let mut binding = db.borrow_mut();
        let name = binding.get_unique_key("new file");
        binding.insert(name.to_owned(), "".to_owned());
        open_file(&mut binding, &mut state.borrow_mut(), ui_handle.clone(), Some(name.to_owned()));
        binding.save(&name);
        drop(binding);
        build_file_tree(db.clone(), model.clone());

    });
}

pub fn handle_close(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_close(move || {
        let ui = ui_handle.unwrap();
        let mut binding = db.borrow_mut();
        ui.window().hide().expect("Failed to hide window");
        println!("Window close requested");
        open_file(&mut binding, &mut state.borrow_mut(), ui_handle.clone(), None);
        binding.save_all().expect("Failed to save all files on exit!");
        state.borrow().save().expect("Failed to save state");
    });
}

pub fn handle_close_popups(ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_close_popups(move || {
        let ui = ui_handle.unwrap();
        ui.invoke_hide_popups();
    });
}

pub fn handle_change_background_image(state: Rc<RefCell<State>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_open_background_image_selection_dialog(move || {
        let ui = ui_handle.unwrap();
        if let Some(path) = FileDialog::new().add_filter("Images",&vec!["png", "jpg"]).pick_file() {
            let p = path.display().to_string().clone();
            let path = Path::new(p.as_str());

            ui.invoke_set_background_image(Image::load_from_path(path).unwrap());
            ui.set_current_background(SharedString::from(p.clone()));
            state.borrow_mut().background_image_path = Some(p.clone());
            return SharedString::from(p);
        } else {
            return SharedString::from("None");
        }

    });
}

pub fn handle_change_dir(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_open_working_directory_selection_dialog(move || {
        let ui = ui_handle.unwrap();
        let mut binding = db.borrow_mut();
        if let Some(path) = FileDialog::new().pick_folder() {
            let p = path.display().to_string().clone();
            println!("Changing directories!");
            open_file(&mut binding, &mut state.borrow_mut(), ui.as_weak(), None);
            binding.change_dirs(p.clone());
            binding.load().expect("Failed to load db");
            println!("Loaded new directory!");
            drop(binding);
            build_file_tree(Rc::clone(&db), model.clone());
            ui.invoke_hide_popups();
            ui.set_current_dir(SharedString::from(p.clone()));
            state.borrow_mut().data_dir = p.clone();

            return SharedString::from(p.clone());
        } else {
            println!("aaaasdfadf\n");
            return SharedString::from("None");
        }

    });
}

pub fn handle_shortcuts(ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_process_shortcut(move |event: KeyEvent| {
        let ui = ui_handle.unwrap();
        if event.modifiers.control && event.text.eq_ignore_ascii_case("t") {
            ui.invoke_show_search_popup();
        }
    });
}

pub fn handle_delete(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, mut model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_delete_file(move |to_delete: SharedString| {
        let mut binding = db.borrow_mut();
        let model = &mut model;
        unsafe {
            if CURRENT_FILE.is_some() {
                let current = CURRENT_FILE.clone().unwrap().to_string();
                if current.as_str() == to_delete.as_str() {
                    open_file(&mut binding, &mut state.borrow_mut(), ui_handle.clone(), None);
                }
            }
            let mut i = 0;
            for  value in model.iter() {
                if value.name.as_str() == to_delete.as_str() {
                    binding.delete_file(value.name.as_str());
                    binding.remove(value.name.as_str());
                    model.remove(i);

                }
                i += 1;
            }
        }();
    });
}


pub fn handle_rename(db: Rc<RefCell<Database>>, mut model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_rename_file(move |new_name: SharedString| {
        let new_name = SharedString::from(remove_invalid_dirs(new_name.to_string()));
        let mut binding = db.borrow_mut();
        let model = &mut model;
        unsafe {
            let current = CURRENT_FILE.clone().unwrap().to_string();
            binding.rename(current.as_str(), new_name.as_str());

            for value in model.iter() {
                if value.full_path.as_str() == current.as_str() {
                    binding.delete_file(current.as_str());


                    println!("renamed to {}", value.name.as_str());

                }
            }
            drop(binding);
            build_file_tree(db.clone(), model.clone());
            CURRENT_FILE = Some(new_name.as_str().to_owned());
        }();
        return new_name.clone();
    });
}
