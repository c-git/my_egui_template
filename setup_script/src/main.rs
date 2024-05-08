const CONFIG_FOLDER_NAME: &str = "egui_setup_from_template";
const CONFIG_FILE_NAME: &str = "config.toml";

use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};

// TODO: Should be more like a wizard and show defaults and let you override because they must change for each project
// TODO: Add LICENSE-APACHE to what gets updated with owner info and dynamic content of the year

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

    let src_path = validate_source_directory(&config.source_path, &config.crate_name.from)?;
    println!("Source: {src_path:?}\nDestination: {dst_path:?}");

    copy_dir::copy_dir(&src_path, &dst_path).context("copy failed")?;

    replace_values(&dst_path, &config).with_context(|| {
        format!("failed to do replacements. Partially setup folder at {dst_path:?}")
    })?;

    delete_setup_files(&dst_path).with_context(|| {
        format!("failed to remove setup files. Partially setup folder at {dst_path:?}")
    })?;

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
    from: String,
    to: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    crate_name: ReplacementPair,
    author_name: ReplacementPair,
    author_email: ReplacementPair,
    struct_name: ReplacementPair,
    app_title: ReplacementPair,
    manifest_name: ReplacementPair,
    manifest_short_name: ReplacementPair,
    source_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crate_name: ReplacementPair {
                from: "eframe_template".into(),
                to: "your_crate".into(),
            },
            author_name: ReplacementPair {
                from: "Emil Ernerfeldt".into(),
                to: "".into(),
            },
            struct_name: ReplacementPair {
                from: "TemplateApp".into(),
                to: "MyApp".into(),
            },
            app_title: ReplacementPair {
                from: "eframe template".into(),
                to: "app title".into(),
            },
            author_email: ReplacementPair {
                from: "emil.ernerfeldt@gmail.com".into(),
                to: "".into(),
            },
            manifest_name: ReplacementPair {
                from: "egui Template PWA".into(),
                to: "my app PWA".into(),
            },
            manifest_short_name: ReplacementPair {
                from: "egui-template-pwa".into(),
                to: "my-app-pwa".into(),
            },
            source_path: "../".into(),
        }
    }
}

fn replace_values(dst_path: &Path, config: &Config) -> anyhow::Result<()> {
    do_replacement(
        dst_path.join("Cargo.toml"),
        vec![
            &config.crate_name,
            &config.author_name,
            &config.author_email,
        ],
    )?;
    do_replacement(dst_path.join("index.html"), vec![&config.app_title])?;
    do_replacement(
        dst_path.join("assets").join("manifest.json"),
        vec![&config.manifest_name, &config.manifest_short_name],
    )?;
    do_replacement(
        dst_path.join("assets").join("sw.js"),
        vec![&config.crate_name, &config.manifest_short_name],
    )?;
    do_replacement(
        dst_path.join("src").join("app.rs"),
        vec![&config.struct_name, &config.app_title],
    )?;
    do_replacement(
        dst_path.join("src").join("lib.rs"),
        vec![&config.struct_name],
    )?;
    do_replacement(
        dst_path.join("src").join("main.rs"),
        vec![&config.struct_name, &config.app_title, &config.crate_name],
    )?;
    Ok(())
}

fn do_replacement(path: PathBuf, replacements: Vec<&ReplacementPair>) -> anyhow::Result<()> {
    let mut content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read contents for {path:?}"))?;
    for replacement in replacements {
        let new_content = content.replace(&replacement.from, &replacement.to);
        if new_content == content {
            eprintln!(
                "Warning: Was not able to find {:?} for replacement in {path:?}",
                replacement.from
            );
        }
        content = new_content;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn delete_setup_files(dst_path: &Path) -> anyhow::Result<()> {
    std::fs::remove_dir_all(dst_path.join(".git")).context("failed to remove .git folder")?;
    std::fs::remove_dir_all(dst_path.join("setup_script"))
        .context("failed to remove setup directory folder")?;
    Ok(())
}
