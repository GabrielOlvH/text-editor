use std::collections::HashMap;
use std::fs::{create_dir, create_dir_all, metadata, OpenOptions, read_dir, remove_file};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use crate::file_types::{FileType, TextFile};

pub(crate) struct Database {
    pub data: HashMap<String, FileType>,
    file_path: String,
    collapsed: Vec<String>
}

impl Database {
    pub fn new(file_path: &str) -> Self {
        if !metadata(&file_path).is_ok() {
            create_dir(&file_path).expect("created cache failed");
            println!("Directory created successfully!");
        } else {
            println!("Directory already exists!");
        }

        Database {
            data: HashMap::new(),
            file_path: file_path.to_string(),
            collapsed: Vec::new()
        }
    }

    pub fn toggle_collapse(&mut self, file: &str) {
        if self.collapsed.contains(&file.to_string()) {
            self.collapsed = self.collapsed.iter().cloned().filter(|s| { *s != file.to_string() }).collect::<Vec<_>>()
        } else {
            self.collapsed.push(file.to_string());
        }
    }

    pub fn collapsed(&self, file: &str) -> bool {
        let a = file.split("/").collect::<Vec<_>>();
        let mut s = String::from("");
        for m in a.clone() {
            s.push_str(m);
            for x in self.collapsed.clone() {
                if x == s.clone() { return true; }
            }
            s.push('/')
        }

        return false;
    }

    pub fn contains(&self, file: &str) -> bool {
        return self.data.contains_key(file);
    }

    pub fn get_file(&mut self, file_name: &str) -> Option<&mut FileType> {
        return self.data.get_mut(file_name);
    }

    pub fn keys(&self) -> Vec<&String> {
        return self.data.keys().collect()
    }

    pub fn change_dirs(&mut self, p: String) {
        self.file_path = p;
        self.data.clear();
    }

    pub fn current_dir(&self) -> String {
        return self.file_path.clone();
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        self.read_dir_recursive(&PathBuf::from(&self.file_path), String::new())?;
        Ok(())
    }

    fn read_dir_recursive(&mut self, path: &Path, prefix: String) -> Result<(), io::Error> {
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let file_name = entry.file_name().into_string().unwrap();

                if path.is_dir() {
                    let new_prefix = if prefix.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}/{}", prefix, file_name)
                    };
                    self.read_dir_recursive(&path, new_prefix)?;
                } else {
                 //   let mut content = String::new();
               //     let mut file = File::open(&path)?;
               //     let r = file.read_to_string(&mut content);
                //    if r.is_err() { continue; }

                    let key = if prefix.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}/{}", prefix, file_name)
                    };
                    self.data.insert(key.clone(), FileType::Text(TextFile { path: path.as_os_str().to_str().unwrap().to_string(), content: None, dirty: false }));
                }
            }
        } else {
            println!("Failed to read directory: {:?}", path);
        }

        Ok(())
    }

    pub fn save_all(&self) -> io::Result<()> {
        for (key, value) in &self.data {

            if let FileType::Text(text_file) = value {
                if !value.is_dirty() { continue; }
                let file_path = format!("{}/{}", &self.file_path, key);
                let path = Path::new(&file_path);

                // Ensure the parent directory exists
                if let Some(parent) = path.parent() {
                    create_dir_all(parent).expect("Failed to create dir");
                }

                // Open the file with write, truncate, and create options
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&file_path).expect(format!("Failed to open file {}", key.clone()).as_str());

                println!("Saved {}", key);

                // Write the value to the file
                if let Some(content) = &text_file.content {
                    writeln!(&mut file, "{}", content.clone()).expect(format!("Failed to save {}", key.clone()).as_str());
                }
            }
        }
        Ok(())
    }

    pub fn delete_file(&self, name: &str) {
        if remove_file(format!("{}/{}", &self.file_path, name)).is_ok() {
            println!("Deleted file {}", name);
        }
    }

    pub fn rename(&mut self, old: &str, new: &str) {
        let contents = self.data.remove(old);
        if contents.is_some() {
            self.data.insert(new.to_owned(), contents.unwrap());
        }
    }

    pub fn remove(&mut self, str: &str) {
        self.data.remove(str);
    }
    pub fn get_unique_key(&self, key: &str) -> String {
        let mut new_key = key.to_string();
        let mut count = 1;
        while self.data.contains_key(&new_key) {
            new_key = format!("{} {}", key, count);
            count += 1;
        }
        new_key
    }

    pub fn save(&self, file: &str) {
        if let FileType::Text(content) = &self.data.get(file).expect("file doesn't exist") {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(format!("{}/{}", &self.file_path, file)).expect("failed to open file");


            if let Some(content) = &content.content {
                writeln!(&mut file, "{}", content).expect("failed to write file");
            }
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.data.insert(key.clone(), FileType::Text(TextFile { path: key.clone(), content: Some(value), dirty: true }));
    }

    pub fn mark_dirty(&mut self, key: String) {
        if let Some(file) = self.data.get_mut(&key) {
            file.set_dirty(true);
        }
    }

    fn delete(&mut self, key: &str) -> Option<FileType> {
        self.data.remove(key)
    }
}