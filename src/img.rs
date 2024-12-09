pub struct Gif {
    pub filepath: String,
    pub info: Vec<gif::Frame<'static>>,
}

impl Gif {
    pub fn new(filepath: &str) -> Self {
        use std::fs::File;
        let input = File::open(filepath).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::Indexed);

        let info: Vec<_> = options
            .read_info(input)
            .unwrap()
            .into_iter()
            .map(|info| info.unwrap())
            .collect();
        Self {
            filepath: filepath.to_string(),
            info,
        }
    }
}
