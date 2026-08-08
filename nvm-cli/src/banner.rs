// nvm-cli/src/banner.rs
//
//! Pretty NVM banners.
use crate::{ansi::unicode_supported, ansiprint};

pub const NVM_ANSI_N_UNICODE: &str = "\x1b[0m\x1b[1m
  ██╗   ██╗███╗   ███╗
  ██║   ██║████╗ ████║
████████████████████████╗
╚══██══██╔═██║═██╔═██║══╝
   ╚████╔╝ ██║ ╚═╝ ██║
    ╚═══╝  ╚═╝     ╚═╝\x1b[0m";

pub const NVM_ANSI_N_ASCII: &str = "\x1b[0m\x1b[1m
  MM    MM MMM    MMM 
  MM    MM MMMM  MMMM  
MMMMMMMMMMMMMMMMMMMMMMMM 
   MM  MM  MM  MM  MM
    MMMM   MM      MM\x1b[0m";

pub fn print_banner() {
    if unicode_supported() {
        ansiprint!("{NVM_ANSI_N_UNICODE}")
    } else {
        ansiprint!("{NVM_ANSI_N_ASCII}")
    }
}
