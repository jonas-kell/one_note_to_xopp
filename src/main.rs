use onenote_parser::Parser;
use onenote_parser::page::{Page, PageContent};
use onenote_parser::contents::{EmbeddedFile, Image, Ink, OutlineElement, RichText, Table, Outline, OutlineItem};
use onenote_parser::contents::Content;
use std::fs::File;
use std::io::prelude::*;
use flate2::GzBuilder;
use flate2::Compression;
use std::fs;
use base64::{engine::general_purpose, Engine as _};

fn main() {
    for element in std::path::Path::new(r"./").read_dir().unwrap() {
        let in_file_path = element.unwrap().path();
        let filename = in_file_path.file_name().unwrap_or_default().to_string_lossy();
        
        if let Some(extension) = in_file_path.extension() {
            if extension == "one" {
                let file_name_without_extension = in_file_path.file_stem().unwrap_or_default().to_string_lossy();
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
            
                // print for debug
                // println!("{}", file_output);
                // export to xml for debug
                fs::write(format!("{}.xml", file_name_without_extension), file_output.clone()).expect("Couldn't write xml file");
                // gzip and export into xopp format
                let f = File::create(format!("{}.xopp", file_name_without_extension)).expect("Couldn't create .xopp file");
                let mut gz = GzBuilder::new()
                                .filename(format!("{}.xml", file_name_without_extension))
                                .comment("Output file")
                                .write(f, Compression::default());
                gz.write_all(&(file_output.into_bytes())).expect("Couldn't write .xopp file");
                gz.finish().expect("Couldn't close .xopp file");
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
        PageContent::Unknown => {
            println!("Page with unknown content");
            String::new()
        }
    }
}

fn render_outline(outline: &Outline) -> String {
    let mut contents = String::new();
    
    for outline_element in flatten_outline_items(outline.items()) {

        if !outline_element.children().is_empty() {
            println!("Outline has children. These are table items which are not being handled");
        }
    
        let outline_content: String = outline_element
            .contents()
            .iter()
            .map(|content| render_outline_content(content))
            .collect();
        contents.push_str(&outline_content);
    }

    return contents;
}

fn flatten_outline_items<'a>(
    items: &'a [OutlineItem]
) -> Box<dyn Iterator<Item = &'a OutlineElement> + 'a> {
    Box::new(items.iter().flat_map(move |item| match item {
        OutlineItem::Element(element) => {
            Box::new(Some(element).into_iter())
        }
        OutlineItem::Group(group) => flatten_outline_items(
            group.outlines(),
        ),
    }))
}

fn render_outline_content(content: &Content) -> String {
    match content {
        Content::RichText(text) => render_rich_text(text),
        Content::Image(image) => render_image(image),
        Content::EmbeddedFile(file) => render_embedded_file(file),
        Content::Table(table) => render_table(table),
        Content::Ink(ink) => render_ink(ink),
        Content::Unknown => {
            println!("Outline with unknown content");
            String::new()
        }
    }
}

fn render_image(image: &Image) -> String {
    let image_base_64: String = general_purpose::STANDARD.encode(&image.data().unwrap_or_default());

    let width= image.layout_max_width().unwrap_or_else(|| 100.0);
    let height = image.layout_max_height().unwrap_or_else(|| 100.0);
    let offset_horizontal = image.offset_horizontal().unwrap_or_else(|| 0.0);
    let offset_vertical = image.offset_vertical().unwrap_or_else(|| 0.0);

    // println!("width:{}", width);
    // println!("height:{}", height);
    // println!("offset_horizontal:{}", offset_horizontal);
    // println!("offset_vertical:{}", offset_vertical);

    let mut image_content = String::from(format!("<image left=\"{}\" top=\"{}\" right=\"{}\" bottom=\"{}\">", 
                                                                page_coordinates(offset_horizontal), 
                                                                page_coordinates(offset_vertical), 
                                                                page_coordinates(offset_horizontal + width), 
                                                                page_coordinates(offset_vertical + height)));
    image_content.push_str(&image_base_64);
    image_content.push_str("</image>\n");

    return image_content;
}

fn render_table(_table: &Table) -> String {
    println!("{}", "Rendering tables not implemented");

    return String::from("");
}

fn render_rich_text(_text: &RichText) -> String {
    println!("{}", "Rendering rich text not implemented");

    return String::from("");
}

fn render_embedded_file(_file: &EmbeddedFile) -> String {
    println!("{}", "Rendering embedded file not implemented");

    return String::from("");
}

fn render_ink(_ink: &Ink) -> String {
    println!("{}", "Render ink");

    return String::from("");
}

fn page_coordinates(inches: f32) -> String {
    format!("{}", (inches * 20.0).round())
}