#[macro_use]
extern crate lazy_static;
use crate::cli::Opt;
use structopt::StructOpt;
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

mod cli;

lazy_static! {
    pub static ref CFG: Opt = Opt::from_args();
}

fn main() {
    let mut nr_processed_files = 0;
    CFG.output_xml; // forces the lazy_static execution and make sure help is displayed 

    for element in std::path::Path::new(r"./").read_dir().unwrap() {
        let in_file_path = element.unwrap().path();
        let filename = in_file_path.file_name().unwrap_or_default().to_string_lossy();
        
        if let Some(extension) = in_file_path.extension() {
            if extension == "one" {
                let file_name_without_extension = in_file_path.file_stem().unwrap_or_default().to_string_lossy();
                println!("Parsing the file {}", filename);

                let mut parser = Parser::new();
                let section = parser.parse_section(&in_file_path).unwrap();
            
                let mut pages_cache = String::new();

                let mut fallback_title_index = 0;
                for page_series in section.page_series() {
                    for page in page_series.pages() {
                        let title = page.title_text().map(|s| s.to_string()).unwrap_or_else(|| {
                            fallback_title_index += 1;

                            format!("Untitled Page {}", fallback_title_index)
                        });

                        let page_name = title.trim().replace("/", "_");
                        let file_name = sanitize_filename::sanitize(format!("{} - {}", file_name_without_extension, page_name));
                        
                        if !CFG.aggregate_pages {
                            output_file(render_page(page), file_name);
                            nr_processed_files += 1;
                        } else {
                            pages_cache.push_str(&render_page(page));
                        }
                    }
                }

                if CFG.aggregate_pages {
                    output_file(pages_cache, String::from(file_name_without_extension));
                    nr_processed_files += 1;
                }
            }
        }
    }

    if nr_processed_files == 0 {
        println!("No '*.one' file found in the execution folder.");
    }
}

fn output_file(page_xml: String, file_name: String) {
    let mut file_output = String::from(
        "<?xml version=\"1.0\" standalone=\"no\"?>
        <xournal creator=\"Xournal++ 1.1.3\" fileversion=\"4\">
        <title>Xournal++ document - see https://github.com/xournalpp/xournalpp</title>\n"
                        );
                    
    file_output.push_str(&page_xml);

    file_output.push_str("</xournal>");

    // export to xml for debug
    if CFG.output_xml {
        fs::write(format!("{}.xml", file_name), file_output.clone()).expect("Couldn't write xml file");
    }

    // gzip and export into xopp format
    let f = File::create(format!("{}.xopp", file_name)).expect("Couldn't create .xopp file");
    let mut gz = GzBuilder::new()
                    .filename(format!("{}.xml", file_name))
                    .comment("Output file")
                    .write(f, Compression::default());
    gz.write_all(&(file_output.into_bytes())).expect("Couldn't write .xopp file");
    gz.finish().expect("Couldn't close .xopp file");
}

fn render_page(page: &Page) -> String {
    let mut page_output = String::new();
    
    let mut size_watcher = SizeWatcher::new();

    let page_content: String = page
        .contents()
        .iter()
        .map(|content| render_page_content(content, &mut size_watcher))
        .collect();

    page_output.push_str(&format!("<page width=\"{}\" height=\"{}\">
        <background type=\"solid\" color=\"#ffffffff\" style=\"graph\"/>
        <layer>\n", size_watcher.size_x + CFG.page_padding, size_watcher.size_y + CFG.page_padding));

    page_output.push_str(&page_content);

    page_output.push_str("</layer>\n</page>\n");

    return page_output;
}

fn render_page_content(content: &PageContent, size_watcher: &mut SizeWatcher) -> String {
    match content {
        PageContent::Outline(outline) => render_outline(outline, size_watcher),
        PageContent::Image(image) => render_image(image, None, size_watcher),
        PageContent::EmbeddedFile(file) => render_embedded_file(file, size_watcher),
        PageContent::Ink(ink) => render_ink(ink, size_watcher),
        PageContent::Unknown => {
            println!("Page with unknown content");
            String::new()
        }
    }
}

fn render_outline(outline: &Outline, size_watcher: &mut SizeWatcher) -> String {
    let mut contents = String::new();
    let outline_offset = (outline.offset_horizontal().unwrap_or(0.0), outline.offset_vertical().unwrap_or(0.0));
    
    for outline_element in flatten_outline_items(outline.items()) {

        if !outline_element.children().is_empty() {
            println!("Outline has children. These are table items which are not being handled");
        }
    
        let outline_content: String = outline_element
            .contents()
            .iter()
            .map(|content| render_outline_content(content, Some(outline_offset), size_watcher))
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

fn render_outline_content(content: &Content, outline_offset: Option<(f32, f32)>, size_watcher: &mut SizeWatcher) -> String {
    match content {
        Content::RichText(text) => render_rich_text(text, size_watcher),
        Content::Image(image) => render_image(image, outline_offset, size_watcher),
        Content::EmbeddedFile(file) => render_embedded_file(file, size_watcher),
        Content::Table(table) => render_table(table, size_watcher),
        Content::Ink(ink) => render_ink(ink, size_watcher),
        Content::Unknown => {
            println!("Outline with unknown content");
            String::new()
        }
    }
}

fn render_image(image: &Image, outline_offset: Option<(f32, f32)>, size_watcher: &mut SizeWatcher) -> String {
    let image_base_64: String = general_purpose::STANDARD.encode(&image.data().unwrap_or_default());

    let width= image.layout_max_width().unwrap_or_else(|| 100.0) * CFG.image_scaling_factor();
    let height = image.layout_max_height().unwrap_or_else(|| 100.0) * CFG.image_scaling_factor();
    let offset_horizontal = image.offset_horizontal().unwrap_or_else(|| 0.0) * CFG.image_offset_factor() + outline_offset.unwrap_or((0.0, 0.0)).0 * CFG.outline_offset_factor();
    let offset_vertical = image.offset_vertical().unwrap_or_else(|| 0.0) * CFG.image_offset_factor() + outline_offset.unwrap_or((0.0, 0.0)).1 * CFG.outline_offset_factor();

    // println!("width:{}", width);
    // println!("height:{}", height);
    // println!("offset_horizontal:{}", offset_horizontal);
    // println!("offset_vertical:{}", offset_vertical);
    // println!("outline_offset_h:{} outline_offset_v:{}", outline_offset.unwrap_or((0.0, 0.0)).0, outline_offset.unwrap_or((0.0, 0.0)).1);

    let mut image_content = String::from(format!("<image left=\"{}\" top=\"{}\" right=\"{}\" bottom=\"{}\">", 
                                                                size_watcher.check_x(offset_horizontal), 
                                                                size_watcher.check_y(offset_vertical), 
                                                                size_watcher.check_x(offset_horizontal + width), 
                                                                size_watcher.check_y(offset_vertical + height)));
    image_content.push_str(&image_base_64);
    image_content.push_str("</image>\n");

    return image_content;
}

fn render_table(_table: &Table, _size_watcher: &mut SizeWatcher) -> String {
    println!("{}", "Rendering tables not implemented");

    return String::from("");
}

fn render_rich_text(_text: &RichText, _size_watcher: &mut SizeWatcher) -> String {
    println!("{}", "Rendering rich text not implemented");

    return String::from("");
}

fn render_embedded_file(_file: &EmbeddedFile, _size_watcher: &mut SizeWatcher) -> String {
    println!("{}", "Rendering embedded file not implemented");

    return String::from("");
}

fn render_ink(ink: &Ink, size_watcher: &mut SizeWatcher) -> String {
    if ink.ink_strokes().is_empty() {
        return String::new();
    }

    let offset_horizontal = ink
            .offset_horizontal()
            .unwrap_or_default();
    let offset_vertical = ink
        .offset_vertical()
        .unwrap_or_default();
    // println!("offset_horizontal: {}, offset_vertical: {}", offset_horizontal, offset_vertical);

    // let display_bounding_box = ink
    //     .bounding_box();
    // let display_y_min = display_bounding_box.map(|bb| bb.y()).unwrap_or_default();
    // let display_x_min = display_bounding_box.map(|bb| bb.x()).unwrap_or_default();
    // println!("display_x_min: {}, display_y_min: {}", display_x_min, display_y_min);
    
    let mut image_content = String::new();
    for ink_stroke in ink.ink_strokes() {
        if ink_stroke.path().len() >= 2 { // no points, will fail
            let color = if let Some(value) = ink_stroke.color() {
                let r = value % 256;
        
                let rem = (value - r) / 256;
                let g = rem % 256;
        
                let rem = (rem - g) / 256;
                let b = rem % 256;
        
                format!("#{:x}{:x}{:x}ff", r, g, b)
            } else {
                "black".to_string()
            };
    
            let width = (1.41 * CFG.ink_width_factor).to_string(); // TODO dynamic e.g. (ink_stroke.width() * INK_WIDTH_SCALING_FACTOR).round().to_string();
    
            image_content.push_str(&format!("<stroke tool=\"{}\" color=\"{}\" width=\"{}\">", "pen", color, width));
    
            let start = ink_stroke.path()[0];
            let mut last_x = start.x() + offset_horizontal * CFG.ink_offset_factor();
            let mut last_y = start.y() + offset_vertical * CFG.ink_offset_factor();
    
            image_content.push_str(&format!("{} {} ", size_watcher.check_x(last_x * CFG.ink_scaling_factor()), size_watcher.check_y(last_y * CFG.ink_scaling_factor())));
            for point in ink_stroke.path()[1..].iter() {
                image_content.push_str(&format!("{} {} ", size_watcher.check_x((last_x + point.x()) * CFG.ink_scaling_factor()), size_watcher.check_y((last_y + point.y()) * CFG.ink_scaling_factor())));
                last_x = last_x + point.x();
                last_y = last_y + point.y();
            }
            if ink_stroke.path().len() < 4 { // if not enough points, will fail
                image_content.push_str(&format!("{} {} ", size_watcher.check_x(last_x * CFG.ink_scaling_factor()), size_watcher.check_y(last_y * CFG.ink_scaling_factor())));
            }
            image_content.push_str("</stroke>\n");
        }
    }

    return image_content;
}

struct SizeWatcher {
    pub size_x: f32,
    pub size_y: f32,
}

impl SizeWatcher {
    pub fn new() -> SizeWatcher {
        SizeWatcher {
            size_x: CFG.a4_page_width,
            size_y: CFG.a4_page_height,
        }
    }

    pub fn check_x (&mut self, x: f32) -> f32 {

        if x > self.size_x {
            self.size_x = x;
        }

        return x; // always return the input for function chaining
    }

    pub fn check_y (&mut self, y: f32) -> f32 {

        if y > self.size_y {
            self.size_y = y;
        }

        return y; // always return the input for function chaining
    }
}