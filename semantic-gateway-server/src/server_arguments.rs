use std::path::PathBuf;

/// Semantic gateway server
#[derive(clap::Parser, Debug)]
#[clap(about, version, long_about = None)]
pub struct ServerArguments {
    /// The directory containing the models
    #[clap(short, long, default_value = "models")]
    models_dir: PathBuf,
}
