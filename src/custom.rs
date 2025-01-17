use crate::{
    input::user::KeyBinding,
    media::{emulator::TerminalEmulator, kitty::KittyMode},
    GraphicsMode,
};
use clap::ValueEnum;
use serde::Deserialize;
use std::{fs, io, path::Path};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,

    #[serde(default)]
    pub typst: TypstConfig,

    #[serde(default)]
    pub options: OptionsConfig,

    #[serde(default)]
    pub bindings: KeyBindingsConfig,
}

impl Config {
    /// Load the config from a path.
    pub fn load(path: &Path) -> Result<Self, ConfigLoadError> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("invalid configuration: {0}")]
    Invalid(#[from] serde_yaml::Error),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultsConfig {
    pub theme: Option<String>,

    #[serde(default = "default_font_size")]
    pub terminal_font_size: u8,

    #[serde(default)]
    pub image_protocol: ImageProtocol,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self { theme: Default::default(), terminal_font_size: default_font_size(), image_protocol: Default::default() }
    }
}

fn default_font_size() -> u8 {
    16
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsConfig {
    /// Whether slides are automatically terminated when a slide title is found.
    pub implicit_slide_ends: Option<bool>,

    /// The prefix to use for commands.
    pub command_prefix: Option<String>,

    /// Show all lists incrementally, by implicitly adding pauses in between elements.
    pub incremental_lists: Option<bool>,

    /// Whether to treat a thematic break as a slide end.
    pub end_slide_shorthand: Option<bool>,

    /// Whether to be strict about parsing the presentation's front matter.
    pub strict_front_matter_parsing: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypstConfig {
    #[serde(default = "default_typst_ppi")]
    pub ppi: u32,
}

impl Default for TypstConfig {
    fn default() -> Self {
        Self { ppi: default_typst_ppi() }
    }
}

fn default_typst_ppi() -> u32 {
    300
}

#[derive(Clone, Debug, Default, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ImageProtocol {
    #[default]
    Auto,
    Iterm2,
    KittyLocal,
    KittyRemote,
    Sixel,
    AsciiBlocks,
}

pub struct SixelUnsupported;

impl TryFrom<&ImageProtocol> for GraphicsMode {
    type Error = SixelUnsupported;

    fn try_from(protocol: &ImageProtocol) -> Result<Self, Self::Error> {
        let mode = match protocol {
            ImageProtocol::Auto => {
                let emulator = TerminalEmulator::detect();
                emulator.preferred_protocol()
            }
            ImageProtocol::Iterm2 => GraphicsMode::Iterm2,
            ImageProtocol::KittyLocal => {
                GraphicsMode::Kitty { mode: KittyMode::Local, inside_tmux: TerminalEmulator::is_inside_tmux() }
            }
            ImageProtocol::KittyRemote => {
                GraphicsMode::Kitty { mode: KittyMode::Remote, inside_tmux: TerminalEmulator::is_inside_tmux() }
            }
            ImageProtocol::AsciiBlocks => GraphicsMode::AsciiBlocks,
            #[cfg(feature = "sixel")]
            ImageProtocol::Sixel => GraphicsMode::Sixel,
            #[cfg(not(feature = "sixel"))]
            ImageProtocol::Sixel => return Err(SixelUnsupported),
        };
        Ok(mode)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyBindingsConfig {
    #[serde(default = "default_next_bindings")]
    pub(crate) next: Vec<KeyBinding>,

    #[serde(default = "default_previous_bindings")]
    pub(crate) previous: Vec<KeyBinding>,

    #[serde(default = "default_first_slide_bindings")]
    pub(crate) first_slide: Vec<KeyBinding>,

    #[serde(default = "default_last_slide_bindings")]
    pub(crate) last_slide: Vec<KeyBinding>,

    #[serde(default = "default_go_to_slide_bindings")]
    pub(crate) go_to_slide: Vec<KeyBinding>,

    #[serde(default = "default_execute_code_bindings")]
    pub(crate) execute_code: Vec<KeyBinding>,

    #[serde(default = "default_reload_bindings")]
    pub(crate) reload: Vec<KeyBinding>,

    #[serde(default = "default_toggle_index_bindings")]
    pub(crate) toggle_slide_index: Vec<KeyBinding>,

    #[serde(default = "default_toggle_bindings_modal_bindings")]
    pub(crate) toggle_bindings: Vec<KeyBinding>,

    #[serde(default = "default_close_modal_bindings")]
    pub(crate) close_modal: Vec<KeyBinding>,

    #[serde(default = "default_exit_bindings")]
    pub(crate) exit: Vec<KeyBinding>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            next: default_next_bindings(),
            previous: default_previous_bindings(),
            first_slide: default_first_slide_bindings(),
            last_slide: default_last_slide_bindings(),
            go_to_slide: default_go_to_slide_bindings(),
            execute_code: default_execute_code_bindings(),
            reload: default_reload_bindings(),
            toggle_slide_index: default_toggle_index_bindings(),
            toggle_bindings: default_toggle_bindings_modal_bindings(),
            close_modal: default_close_modal_bindings(),
            exit: default_exit_bindings(),
        }
    }
}

fn make_keybindings<const N: usize>(raw_bindings: [&str; N]) -> Vec<KeyBinding> {
    let mut bindings = Vec::new();
    for binding in raw_bindings {
        bindings.push(binding.parse().expect("invalid binding"));
    }
    bindings
}

fn default_next_bindings() -> Vec<KeyBinding> {
    make_keybindings(["l", "j", "<right>", "<page_down>", "<down>", " "])
}

fn default_previous_bindings() -> Vec<KeyBinding> {
    make_keybindings(["h", "k", "<left>", "<page_up>", "<up>"])
}

fn default_first_slide_bindings() -> Vec<KeyBinding> {
    make_keybindings(["gg"])
}

fn default_last_slide_bindings() -> Vec<KeyBinding> {
    make_keybindings(["G"])
}

fn default_go_to_slide_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<number>G"])
}

fn default_execute_code_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<c-e>"])
}

fn default_reload_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<c-r>"])
}

fn default_toggle_index_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<c-p>"])
}

fn default_toggle_bindings_modal_bindings() -> Vec<KeyBinding> {
    make_keybindings(["?"])
}

fn default_close_modal_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<esc>"])
}

fn default_exit_bindings() -> Vec<KeyBinding> {
    make_keybindings(["<c-c>"])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::input::user::CommandKeyBindings;

    #[test]
    fn default_bindings() {
        let config = KeyBindingsConfig::default();
        CommandKeyBindings::try_from(config).expect("construction failed");
    }
}
