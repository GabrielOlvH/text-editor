use std::fs::File;
use std::io::Read;

pub enum FileType {
    Text(TextFile),
    Image(ImageFile),

}

pub struct TextFile {
    pub path: String,
    pub content: Option<String>,
    pub dirty: bool
}

pub struct ImageFile {
    pub path: String,
    pub dirty: bool
}

impl FileType {
    pub fn display(&self) {
        match self {
            FileType::Text(file) => file.display(),
            FileType::Image(file) => file.display(),
        }
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        match self {
            FileType::Text(file) => file.set_dirty(dirty),
            FileType::Image(file) => file.set_dirty(dirty),
        }
    }


    pub fn get_path(&mut self) -> String {
        match self {
            FileType::Text(file) => file.path.clone(),
            FileType::Image(file) => file.path.clone(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        match self {
            FileType::Text(file) => file.is_dirty(),
            FileType::Image(file) => file.is_dirty(),
        }
    }
}
impl TextFile {
    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn display(&self) {
        println!("Displaying text file: {:?}", self.content);
    }

    pub fn get_contents(&mut self) -> String {
        if let Some(content) = &self.content {
            return content.clone();
        } else {
            let mut content = String::new();
            let mut file = File::open(&self.path).expect(format!("File {} not found", &self.path.clone()).as_str());
            let r = file.read_to_string(&mut content);
            if r.is_err() { return String::from("???"); }

            self.content = Some(content.clone());
            return content.clone();
        }
    }
}

impl ImageFile {
    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    fn display(&self) {
        println!("Displaying image file from path: {}", self.path);
    }
}
