use bkrbk::win;

pub mod img;
pub mod timer;

#[derive(Debug)]
enum Error {
    ParseArgs,
    ParseInfo,
}

fn parse_args() -> Result<(String, u32), Error> {
    let args = std::env::args().collect::<Vec<String>>();
    let path = args.get(1).cloned().ok_or(Error::ParseArgs)?;
    // let bg_color = args.get(2).cloned().ok_or(Error::ParseArgs)?;
    Ok((path, u32::from_str_radix("FF0000", 16).unwrap()))
}

fn get_image_size(image: &img::Gif) -> Result<(u16, u16), Error> {
    let wframe = image
        .info
        .iter()
        .max_by_key(|info| info.top + info.width)
        .ok_or(Error::ParseInfo)?;
    let width = wframe.top + wframe.width;

    let wframe = image
        .info
        .iter()
        .max_by_key(|info| info.top + info.height)
        .ok_or(Error::ParseInfo)?;
    let height = wframe.top + wframe.height;

    Ok((width, height))
}

pub struct Counter {
    max: usize,
    value: usize,
}

impl Counter {
    pub fn new(max: usize) -> Self {
        Self { max, value: 0 }
    }

    pub fn update(&mut self) -> usize {
        if self.value < self.max {
            let result = self.value;
            self.value += 1;
            result
        } else {
            let result = 0;
            self.value = 1;
            result
        }
    }
}

const DEFAULT_BG: u32 = 0x00_FF_FF_FF;

fn main() {
    let (filepath, bg_color) = parse_args().unwrap();
    let image = img::Gif::new(&filepath);
    let (width, height) = get_image_size(&image).unwrap();

    let mut core = win::Core::new("bkrbk", width as _, height as _, bg_color);
    core.init().unwrap();
    core.reset();
    core.front.show(
        core.handler,
        core.class_name.reference,
        &core.main_window_name.entity,
    );

    let mut timer = timer::Timer::new(1);
    let mut current = Counter::new(image.info.len());

    // main loop
    'main: loop {
        match core.update() {
            None => break 'main,
            Some(_) => {
                let c = current.update();
                let info = image.info.get(c).unwrap(); // TODO: unwrap() をなくす

                // 描画
                let palette = {
                    let palette = info.palette.clone().unwrap();
                    let mut iter = palette.into_iter();
                    let mut result = vec![DEFAULT_BG; u8::MAX as usize];
                    let mut count = 0;
                    while let Some(color) = iter.next() {
                        let r = color;
                        let g = iter.next().unwrap();
                        let b = iter.next().unwrap();
                        let color = ((r as u32) << 16) + ((g as u32) << 8) + (b as u32);
                        result[count] = color;
                        count += 1;
                    }
                    result
                };

                for y in 0..info.height as usize {
                    for x in 0..info.width as usize {
                        let base_i = y * info.width as usize + x;
                        let core_i =
                            (y + info.top as usize) * width as usize + x + info.left as usize;
                        let pale_i = info.buffer[base_i];

                        // 0の場合は上書きしない
                        if c == 0 || 0 != pale_i {
                            let color = if bg_color == palette[pale_i as usize] {
                                // bgによってハンドリングしたい場合はここを変える
                                // DEFAULT_BG
                                palette[pale_i as usize]
                            } else {
                                palette[pale_i as usize]
                            };
                            // core.image[core_i] = color;
                            core.front.image[core_i] = color;
                        }
                    }
                }
                // core.front.image = core.image.clone();
                // core.draw();
                // core.call_draw();
                core.front.draw();
                core.front.call_draw();

                let millis = info.delay * 10;
                timer.interval = std::time::Duration::from_millis(millis as u64);
                timer.sleep();
            }
        }
    }
}
