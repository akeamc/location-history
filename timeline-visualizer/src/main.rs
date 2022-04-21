use std::{
    fs::{create_dir_all, File},
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};

use colorgrad::Gradient;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_cross_mut;
use location_history::{protocol::LocationEntry, read_json_entries};
use structopt::StructOpt;
use timeline_visualizer::projection::CroppedWebMercator;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, default_value = "out", parse(from_os_str))]
    output: PathBuf,
}

struct View {
    projection: CroppedWebMercator,
    image: RgbImage,
    gradient: Gradient,
}

impl View {
    pub fn from_projection(projection: CroppedWebMercator) -> Self {
        let mut image = RgbImage::new(projection.width(), projection.height());

        image.fill(255);

        Self {
            image,
            projection,
            gradient: colorgrad::rainbow(),
        }
    }

    pub fn put_entry(&mut self, entry: &LocationEntry) {
        if let Some((x, y)) = self.projection.project_int(entry.lnglat()) {
            let time = entry.timestamp().time();
            let seconds =
                3600 * time.hour() as u32 + 60 * time.minute() as u32 + time.second() as u32;
            let color = self.gradient.at(seconds as f64 / 86400.0);
            let color = Rgb([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ]);
            draw_cross_mut(&mut self.image, color, x as _, y as _);
        }
    }

    pub fn image(&self) -> &RgbImage {
        &self.image
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args_safe()?;

    let output_dir = Path::new(&opt.output);
    let records = BufReader::new(File::open(&opt.input)?);

    let mut views = vec![
        View::from_projection(CroppedWebMercator::new(
            1920, 1080, 18.0117, 18.1208, 59.2967,
        )), // Södermalm
        View::from_projection(CroppedWebMercator::new(
            3840, 2160, 17.7499, 18.3808, 59.2226,
        )), // Stockholm
        View::from_projection(CroppedWebMercator::new(2160, 3840, 8.296, 25.5124, 54.2021)), // Sweden
        View::from_projection(CroppedWebMercator::new(
            3840, 2160, -133.1357, 35.958, 20.8103,
        )), // North Atlantic
    ];

    println!("reading {}", opt.input.display());

    let mut count = 0;
    let before = Instant::now();

    read_json_entries(records, |entry| {
        for view in views.iter_mut() {
            view.put_entry(&entry);
            count += 1;
        }
    })?;

    let elapsed = before.elapsed();
    println!(
        "read {count} entries in {:.02?} ({:.02} entries/s)",
        elapsed,
        count as f32 / elapsed.as_secs_f32()
    );

    eprintln!("saving images to {}", output_dir.display());

    create_dir_all(output_dir)?;

    for (i, view) in views.iter().enumerate() {
        view.image().save(output_dir.join(format!("{i}.png")))?;
    }

    Ok(())
}
