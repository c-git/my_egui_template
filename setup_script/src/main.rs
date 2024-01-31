const CONFIG_FOLDER_NAME: &str = "egui_setup_from_template";
const CONFIG_FILE_NAME: &str = "config.toml";

use std::{io::Write as _, path::PathBuf};

use anyhow::{bail, Context};

fn main() -> anyhow::Result<()> {
    use clap::Parser as _;
    let cli = cli::Cli::parse();
    if cli.generate_config {
        return generate_config();
    }
    let Some(dst_path) = cli.dst_path else {
        unreachable!("Should be ensured be settings for clap")
    };
    let dst_path = PathBuf::from(dst_path);

    // Load config
    let config = if let Some(config_path) = cli.config_path {
        // Load from user supplied config file
        load_config(PathBuf::from(&config_path))?
    } else {
        // Try to load from default
        let default_config_path = config_path()?;
        load_config(default_config_path)?
    };

    let src_path = validate_source_directory(&config.source_path, &config.crate_.src)?;
    println!("Source: {src_path:?}\nDestination: {dst_path:?}");

    copy_dir::copy_dir(&src_path, &dst_path).context("copy failed")?;

    println!("Completed successfully");
    Ok(())
}

mod cli {
    use clap::Parser;

    #[derive(Parser, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
    #[command(
        version,
        about,
        long_about = "
Create a new egui project from the template. 

- Requires the template be cloned locally and the path to it is available.
"
    )]

    /// Stores the configurations acquired via the command line
    pub struct Cli {
        #[arg(value_name = "TARGET_PATH", required_unless_present_any(["generate_config"]))]
        /// The new folder to be created (Must NOT exist)
        pub dst_path: Option<String>,

        /// The folder to copy from must exist and is expected to be the template
        pub config_path: Option<String>,

        /// Generate a new config file if it doesn't already exist
        #[arg(long)]
        pub generate_config: bool,
    }
    #[cfg(test)]
    mod tests {

        #[test]
        fn verify_cli() {
            // Source: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing
            // My understanding it reports most development errors without additional effort
            use clap::CommandFactory;
            super::Cli::command().debug_assert()
        }
    }
}

fn validate_source_directory(source_path: &str, src_crate_name: &str) -> anyhow::Result<PathBuf> {
    #[derive(serde::Deserialize)]
    struct Package {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct CargoToml {
        package: Package,
    }
    let path = PathBuf::from(source_path);
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize source path: {source_path:?}"))?;

    // Check for `Cargo.toml` and validate source crate name
    let cargo_toml = path.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to open Cargo.toml from: {cargo_toml:?}"))?;
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml).with_context(|| {
        format!("failed to parse Cargo.toml from source directory: {cargo_toml:?}")
    })?;
    if cargo_toml.package.name != src_crate_name {
        bail!("validation of source directory failed. Expected source package name to be {src_crate_name:?} but found {:?}", cargo_toml.package.name)
    }
    Ok(path)
}

fn load_config(config_path: PathBuf) -> anyhow::Result<Config> {
    let config_path = config_path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize the path received {config_path:?}"))?;
    let toml = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to config read from {config_path:?}"))?;
    let result = toml::from_str(&toml)
        .with_context(|| format!("failed to parse toml file: {config_path:?}"))?;
    Ok(result)
}

fn generate_config() -> anyhow::Result<()> {
    let config_path = config_path()?;
    let config = Config::default();
    let Some(parent_dir) = config_path.parent() else {
        bail!("Expected config to have a parent directory but none was found. Config path: {config_path:?}")
    };
    std::fs::create_dir_all(parent_dir).context("failed to create directory for config")?;
    let toml = toml::to_string(&config).context("failed to convert to toml")?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&config_path)
        .with_context(|| format!("failed to create config file: {config_path:?}"))?;
    file.write_all(toml.as_bytes())
        .with_context(|| format!("failed to write to {config_path:?}"))?;
    println!("Config written to {config_path:?}");
    Ok(())
}

fn config_path() -> anyhow::Result<PathBuf> {
    let Some(config_path) = dirs::config_dir() else {
        bail!("Unable to determine config directory")
    };
    let result = config_path.join(CONFIG_FOLDER_NAME).join(CONFIG_FILE_NAME);
    Ok(result)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ReplacementPair {
    src: String,
    dst: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    #[serde(rename = "crate")]
    crate_: ReplacementPair,
    author_name: ReplacementPair,
    author_email: ReplacementPair,
    source_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_path: "../".into(),
            crate_: ReplacementPair {
                src: "eframe_template".into(),
                dst: "your_crate".into(),
            },
            author_name: ReplacementPair {
                src: "Emil Ernerfeldt".into(),
                dst: "".into(),
            },
            author_email: ReplacementPair {
                src: "emil.ernerfeldt@gmail.com".into(),
                dst: "".into(),
            },
        }
    }
}
