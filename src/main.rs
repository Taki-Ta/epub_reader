use roxmltree::Document;
use serde::Serialize;
use zip::read::ZipArchive;

use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize)]
struct EpubContent {
    title: String,
    content: String,
}

#[allow(unused)]
fn load_epub(file_path: &str) -> Result<EpubContent, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Find the content.opf file
    let mut content_opf = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        println!("Filename: {}", file.name());
        std::io::copy(&mut file, &mut std::io::stdout()).unwrap();
    }
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().ends_with("content.opf") {
            file.read_to_string(&mut content_opf)
                .map_err(|e| e.to_string())?;
            break;
        }
    }

    // Parse content.opf
    let doc = Document::parse(&content_opf).map_err(|e| e.to_string())?;
    let mut content = String::new();
    let mut title = String::new();

    // Find and read the first HTML file listed in the content.opf
    if let Some(item) = doc.descendants().find(|n| {
        n.has_tag_name("item") && n.attribute("media-type") == Some("application/xhtml+xml")
    }) {
        let href = item.attribute("href").unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            if file.name().ends_with(href) {
                file.read_to_string(&mut content)
                    .map_err(|e| e.to_string())?;
                break;
            }
        }
    }

    // Find the title
    if let Some(title_elem) = doc.descendants().find(|n| n.has_tag_name("title")) {
        title = title_elem.text().unwrap_or("").to_string();
    }

    Ok(EpubContent { title, content })
}

// slint::include_modules!();

fn main() {
    // let main_window = MainWindow::new();
    // let epub_content = Rc::new(RefCell::new(EpubContent { title: String::new(), content: String::new() }));

    // {
    //     let epub_content = Rc::clone(&epub_content);
    //     main_window.on_load_epub(move |file_path| {
    //         let result = load_epub(&file_path.to_string());
    //         if let Ok(content) = result {
    //             let mut epub_content = epub_content.borrow_mut();
    //             epub_content.title = content.title.clone();
    //             epub_content.content = content.content.clone();

    //             main_window.set_title(SharedString::from(content.title));
    //             main_window.set_content(SharedString::from(content.content));
    //         } else {
    //             main_window.set_title(SharedString::from("Error loading EPUB"));
    //             main_window.set_content(SharedString::from("Failed to load EPUB file"));
    //         }
    //     });
    // }

    // main_window.run();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_load_epub() {
//         let epub = load_epub("assets/book.epub");
//         println!("{:?}", epub);
//         assert!(epub.is_ok());
//         let epub = epub.unwrap();
//         assert_eq!(epub.title, "Test EPUB");
//         assert!(epub.content.contains("Test EPUB"));
//     }
// }
