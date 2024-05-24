use std::cell::RefCell;
use std::rc::Rc;
use slint::{Model, SharedString, VecModel, Weak};
use crate::{AppWindow, build_file_tree, CURRENT_FILE, FileTreeItemData, open_file, remove_invalid_dirs};
use crate::cache::Database;



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

pub fn handle_click_file_tree(db: Rc<RefCell<Database>>, model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_clicked(move |index: i32| {
        let mut binding = db.borrow_mut();
        let opt = model.row_data(index.try_into().unwrap());
        if opt.is_some() {

            let item = opt.unwrap();
            println!("Clicked on {}", item.full_path);
            if binding.contains(item.full_path.as_str()) {
                open_file(&mut binding, ui_handle.clone(), Some(item.full_path.to_string()));
            } else {
                binding.toggle_collapse(item.full_path.as_str());
                println!("Collapsed {}", item.full_path);
                drop(binding);
                build_file_tree(db.clone(), model.clone());
            }
        }
    });
}

pub fn handle_new_file_button(db: Rc<RefCell<Database>>, model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_new_file(move || {
        let mut binding = db.borrow_mut();
        let name = binding.get_unique_key("new file");
        binding.insert(name.to_owned(), "".to_owned());
        open_file(&mut binding, ui_handle.clone(), Some(name.to_owned()));
        binding.save(&name);
        drop(binding);
        build_file_tree(db.clone(), model.clone());

    });
}

pub fn handle_delete(db: Rc<RefCell<Database>>, mut model: Rc<VecModel<FileTreeItemData>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_delete_file(move |to_delete: SharedString| {
        let mut binding = db.borrow_mut();
        let model = &mut model;
        unsafe {
            if CURRENT_FILE.is_some() {
                let current = CURRENT_FILE.clone().unwrap().to_string();
                if current.as_str() == to_delete.as_str() {
                    open_file(&mut binding, ui_handle.clone(), None);
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
                    CURRENT_FILE = Some(new_name.as_str().to_owned());
                }
            }
            drop(binding);
            build_file_tree(db.clone(), model.clone());
        }();
        return new_name.clone();
    });
}
