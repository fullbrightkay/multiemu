use clap::Subcommand;
use native::NativeAction;
use nointro::NoIntroAction;

pub mod native;
pub mod nointro;
pub mod screenscraper;

#[derive(Clone, Debug, Subcommand)]
pub enum DatabaseAction {
    NoIntro {
        #[clap(subcommand)]
        action: NoIntroAction,
    },
    Native {
        #[clap(subcommand)]
        action: NativeAction,
    },
    ScreenScraper {},
}
