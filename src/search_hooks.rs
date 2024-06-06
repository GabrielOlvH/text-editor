use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;
use regex::Regex;
use slint::{ComponentHandle, Model, SharedString, Weak};
use crate::{AppWindow, open_file, SearchResult};
use crate::cache::Database;
use crate::state::State;

pub fn on_pressed_enter(db: Rc<RefCell<Database>>, state: Rc<RefCell<State>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_enter_callback(move || {
        println!("enter!!");
        let ui = ui_handle.unwrap();

        let current_results = ui.invoke_get_search_results();
        for x in 0..current_results.row_count() {
            let result = current_results.row_data(x).unwrap();
            if result.selected {
                ui.invoke_hide_popups();
                open_file(&mut db.borrow_mut(), &mut state.borrow_mut(), ui.as_weak(), Some(result.file_path.to_string()));
                ui.invoke_highlight(result.start, result.end);
                break;
            }
        }
    });
}

pub fn on_move_down(db: Rc<RefCell<Database>>, ui_handle: Weak<AppWindow>) {
    ui_handle.unwrap().on_move_down_result(move || {

        let ui = ui_handle.unwrap();

        let current_results = ui.invoke_get_search_results();
        let mut found = false;
        for mut x in 0..current_results.row_count() {
            let mut result = current_results.row_data(x).unwrap();
            if result.selected {
                found = true;
                result.selected = false;
                current_results.set_row_data(x, result.clone());
                if x + 1 >= current_results.row_count() {
                    x = 0;
                    result = current_results.row_data(0).unwrap();
                } else {
                    continue;
                }
            }

            if (found) {

                println!("{} >= {}", x+1, current_results.row_count());
                result.selected = true;
                println!("new selected is {}", result.file_path);
                current_results.set_row_data(x, result.clone());
                return x as i32;
            }
        }
        return 0;
    });
}

pub fn on_search(db: Rc<RefCell<Database>>, ui_handle: Weak<AppWindow>) {

    ui_handle.unwrap().on_search(move |match_name: bool, match_contents: bool, match_case: bool, regex: bool, terms: SharedString| {
        if terms.is_empty() {
            return;
        }

        let ui = ui_handle.unwrap();
        let mut binding = db.borrow_mut();
        let mut terms = terms.to_string();
        let mut results: Vec<SearchResult> = Vec::new();
        let mut selected_first = false;


        if !match_case {
            terms = terms.to_lowercase().to_string();
        }

        for key in binding.keys() {
            if match_contents {
                let mut contents = binding.get_contents(key).to_string();

                if !match_case {
                    contents = contents.to_lowercase();
                }
                let opt = find(regex, terms.to_string(), contents.to_string());

                if opt.is_some() {
                    let start = opt.unwrap();
                    let snippet_start = max((start as i32) - 8, 0) as usize;
                    let snippet_end = min(start + terms.len() + 8, contents.len());
                    let mut snippet = &mut contents[snippet_start..snippet_end].replace("\n", "\\n");

                    if snippet_start > 0 {
                        snippet.insert_str(0, "...");
                    }
                    if snippet_end < contents.len() {
                        snippet.push_str("...");
                    }
                    results.push(SearchResult {
                        file_path: SharedString::from(key),
                        line_matched: SharedString::from(snippet.clone()),
                        match_name: false,
                        match_contents: true,
                        selected: !selected_first,
                        start: start as i32,
                        end: start as i32 + terms.len() as i32
                    });
                    selected_first = true;
                }
            }

            if match_name {
                let opt = find(regex, terms.to_string(), key.to_string());
                if opt.is_some() {
                    results.push(SearchResult {
                        file_path: SharedString::from(key),
                        line_matched: SharedString::from(""),
                        match_name: true,
                        match_contents: false,
                        selected: !selected_first,
                        start: 0,
                        end: 0
                    });

                    selected_first = true;
                }
            }
        }
        let rc = Rc::new(slint::VecModel::from(vec![]));
        rc.set_vec(results.clone());
        ui.invoke_set_search_results(rc.into());
        println!("Matches found {}", results.clone().len())
    });
}


fn find(regex: bool, terms: String, str: String) -> Option<usize> {
    if !regex {
        return str.find(terms.as_str());
    }
    let reg = Regex::new(terms.as_str());
    return if !reg.is_ok() {
        None
    } else {
        let find = reg.unwrap().find(str.as_str());
        if find.is_some() {
            Some(find.unwrap().start())
        } else {
            None
        }
    }
}