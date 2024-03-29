use crate::{FROM_OS_STRING_FAILED, IMPROPER_HEX_FORMAT, NO_HOME, PROGRAM_DESC, PROGRAM_NAME};
use anyhow::Context;
use args::{
    validations::{Order, OrderValidation},
    Args,
};
use getopts::Occur;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{Read, Write},
};
use tui::{
    style,
    style::{Color, Modifier},
};

#[derive(Serialize, Deserialize, Default)]
pub struct Style {
    pub fg: Option<String>,
    pub bg: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub volume: f64,
    pub detail: f64,
    pub fps: Option<usize>,
    pub style: Style,
}

impl Style {
    fn decode_style_value(hex: &Option<String>) -> anyhow::Result<Option<Color>> {
        match hex {
            Some(v) => {
                if v.len() != 7 || !v.starts_with('#') {
                    return Err(anyhow::Error::msg(IMPROPER_HEX_FORMAT));
                }

                let decoded = hex::decode(&v[1..7])?;

                Ok(Some(Color::Rgb(decoded[0], decoded[1], decoded[2])))
            }
            None => Ok(None),
        }
    }

    pub fn to_tui_style(&self) -> anyhow::Result<style::Style> {
        Ok(style::Style {
            fg: Self::decode_style_value(&self.fg)?,
            bg: Self::decode_style_value(&self.bg)?,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        })
    }
}

impl Config {
    fn generate_default_config_pretty() -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(&Self::default())?)
    }

    pub fn print_default_config() -> anyhow::Result<()> {
        println!("{}", Self::generate_default_config_pretty()?);

        Ok(())
    }

    pub fn try_create_default_config_file() -> anyhow::Result<()> {
        let directory_path = home_dir().context(NO_HOME)?.join(".config/mvis");
        let file_path = directory_path.join("config.json");

        if !file_path.exists() {
            create_dir_all(directory_path)?;

            File::create(file_path)?
                .write_all(Self::generate_default_config_pretty()?.as_bytes())?
        }

        Ok(())
    }

    pub fn create_args() -> anyhow::Result<Args> {
        let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);

        args.flag("h", "help", "Print the usage menu.");
        args.flag(
            "r",
            "regenerate-config",
            "Print the default config to standard output.",
        );
        args.option(
            "F",
            "file",
            "The path to the audio file.",
            "FILE",
            Occur::Optional,
            None,
        );
        args.option(
            "c",
            "config",
            "The path to the config file.",
            "CONFIG",
            Occur::Optional,
            Some(
                home_dir()
                    .context(NO_HOME)?
                    .join(".config/mvis/config.json")
                    .into_os_string()
                    .into_string()
                    .map_err(|_| anyhow::Error::msg(FROM_OS_STRING_FAILED))?,
            ),
        );
        args.option(
            "v",
            "volume",
            "Sets the volume.",
            "VOLUME",
            Occur::Optional,
            None,
        );
        args.option(
            "d",
            "detail",
            "The detail in each frame.",
            "DETAIL",
            Occur::Optional,
            None,
        );
        args.option(
            "f",
            "fps",
            "The target frames per second.",
            "FRAMES_PER_SECOND",
            Occur::Optional,
            None,
        );
        args.option(
            "b",
            "bar-width",
            "The width of the bars.",
            "BAR_WIDTH",
            Occur::Optional,
            None,
        );

        args.parse(env::args())?;

        Ok(args)
    }

    fn from_config(path: String) -> anyhow::Result<Self> {
        let mut contents = String::new();

        File::open(path)?.read_to_string(&mut contents)?;

        Ok(serde_json::from_str(contents.as_str())?)
    }

    pub fn from_arguments(args: &Args) -> anyhow::Result<Self> {
        let mut config = Self::from_config(args.value_of("config")?)?;

        if let Ok(volume) = args.validated_value_of(
            "volume",
            &[
                Box::new(OrderValidation::new(Order::GreaterThanOrEqual, 0_f64)),
                Box::new(OrderValidation::new(Order::LessThanOrEqual, 1_f64)),
            ],
        ) {
            config.volume = volume;
        }

        if let Ok(detail) = args.validated_value_of(
            "detail",
            &[
                Box::new(OrderValidation::new(Order::GreaterThanOrEqual, 0_f64)),
                Box::new(OrderValidation::new(Order::LessThanOrEqual, 1_f64)),
            ],
        ) {
            config.detail = detail;
        }

        if let Ok(fps) = args.validated_value_of(
            "fps",
            &[Box::new(OrderValidation::new(Order::GreaterThanOrEqual, 0))],
        ) {
            config.fps = Some(fps);
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 1_f64,
            detail: 0.1,
            fps: Some(60),
            style: Style::default(),
        }
    }
}
