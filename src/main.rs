use onenote_parser::Parser;
use onenote_parser::page::{Page, PageContent};
use onenote_parser::contents::{Outline, EmbeddedFile, Image, Ink};
use std::fs;

fn main() {
    for element in std::path::Path::new(r"./").read_dir().unwrap() {
        let in_file_path = element.unwrap().path();
        let filename = in_file_path.file_name().unwrap_or_default().to_string_lossy();
        
        if let Some(extension) = in_file_path.extension() {
            
            if extension == "one" {
                println!("Parsing the file {}", filename);

                let mut parser = Parser::new();
            
                let section = parser.parse_section(&in_file_path).unwrap();
            
                let mut file_output = String::from(
"<?xml version=\"1.0\" standalone=\"no\"?>
<xournal creator=\"Xournal++ 1.1.3\" fileversion=\"4\">
<title>Xournal++ document - see https://github.com/xournalpp/xournalpp</title>\n"
                );
            
                let mut fallback_title_index = 0;
                for page_series in section.page_series() {
                    for page in page_series.pages() {
                        let title = page.title_text().map(|s| s.to_string()).unwrap_or_else(|| {
                            fallback_title_index += 1;
            
                            format!("Untitled Page {}", fallback_title_index)
                        });
            
                        let file_name = title.trim().replace("/", "_");
                        println!("{}", file_name);
            
                        file_output.push_str(&render_page(page));
                    }
                }
            
                file_output.push_str("</xournal>");
            
                println!("{}", file_output);
            
                fs::write("out.xml", file_output).expect("Couldn't write file");
            }
        }
    }
}

pub(crate) fn render_page(page: &Page) -> String {
    let mut page_output = String::from(
"<page width=\"595.27559100\" height=\"841.88976400\">
<background type=\"solid\" color=\"#ffffffff\" style=\"graph\"/>
<layer>\n"
            );

    let page_content: String = page
        .contents()
        .iter()
        .map(|content| render_page_content(content))
        .collect();
    page_output.push_str(&page_content);

    page_output.push_str("</layer>\n</page>\n");

    return page_output;
}

fn render_page_content(content: &PageContent) -> String {
    match content {
        PageContent::Outline(outline) => render_outline(outline),
        PageContent::Image(image) => render_image(image),
        PageContent::EmbeddedFile(file) => render_embedded_file(file),
        PageContent::Ink(ink) => render_ink(ink),
        PageContent::Unknown => String::new(),
    }
}

fn render_outline(_outline: &Outline) -> String {
    println!("{}", "Rendering Outline not implemented");

    return String::from("");
}

fn render_image(_outline: &Image) -> String {
    println!("{}", "Render image");

    return String::from("");
}

fn render_embedded_file(_outline: &EmbeddedFile) -> String {
    println!("{}", "Rendering embedded file not implemented");

    return String::from("");
}

fn render_ink(_outline: &Ink) -> String {
    // println!("{}", "render ink");

    return String::from("");
}