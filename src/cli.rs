use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "one_note_to_xopp",
    about = "\nHi, I'm the .one to .xopp transpiler. \nI'm goint to transcode all .one files in the execution location into .xopp."
)]
pub struct Opt {
    #[structopt(
        short,
        long,
        help = "By default pages get split. If passed, the pages will be aggregated into one output file"
    )]
    pub aggregate_pages: bool,

    #[structopt(
        short,
        long,
        default_value = "1.0",
        help = "Modify the overall output scaling"
    )]
    pub scale: f32,

    #[structopt(short = "x", long, help = "Output the xml pre-files for debug")]
    pub output_xml: bool,

    #[structopt(
        short = "p",
        long,
        default_value = "30.0",
        help = "Padding on the sides of the pages"
    )]
    pub page_padding: f32,

    #[structopt(
        short = "iw",
        long,
        default_value = "1.0",
        help = "Modify the width of the ink lines"
    )]
    pub ink_width_factor: f32,

    #[structopt(
        short = "W",
        long,
        default_value = "595.27559100",
        help = "Default min width of one page"
    )]
    pub a4_page_width: f32,

    #[structopt(
        short = "H",
        long,
        default_value = "841.88976400",
        help = "Default min height of one page"
    )]
    pub a4_page_height: f32,
}

impl Opt {
    pub fn outline_offset_factor(&self) -> f32 {
        self.scale * 20.0 // fixed (fit parameter)
    }
    pub fn image_scaling_factor(&self) -> f32 {
        self.scale * 20.0 // fixed (fit parameter)
    }
    pub fn image_offset_factor(&self) -> f32 {
        self.scale * 20.0 // fixed (fit parameter)
    }
    pub fn ink_scaling_factor(&self) -> f32 {
        self.scale * 16.0 / 1000.0 // fixed (fit paramter)
    }
    pub fn ink_offset_factor(&self) -> f32 {
        1270.0 // fixed (fit paramter)
    }
}
