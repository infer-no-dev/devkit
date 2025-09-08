// Button.rs

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Button {
    pub id: String,
    pub label: String,
    pub onClick: mpsc::Sender<String>,
}

impl Button {
    pub fn new(id: String, label: String) -> Self {
        let (tx, rx) = mpsc::channel();
        Button {
            id,
            label,
            onClick: tx,
        }
    }

    pub async fn click(&self) {
        self.onClick.send(self.id.clone()).unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let path = PathBuf::from(temp_dir.path());
    fs::write(path.join("index.html"), "<html><body></body></html>")?;
    env::set_current_dir(path)?;

    // Initialize Button instance
    let button = Button::new(String::from("my-button"), String::from("Click me"));

    // Create React component with useState hook
    use react_core::{Component, Element};
    use react_core::hooks::useState;
    pub fn MyButton() -> Component {
        let (label, setLabel) = useState(String::from("Initial label"));
        button.onClick.clone();

        react_core::html! {
            <button onclick={|e: Event| {
                e.prevent_default();
                button.click().await;
                setLabel(String::from("Button clicked!"));
            }}>
                { label }
            </button>
        }
    }

    MyButton
}