# egui Template copy and setup script

Simple program to copy the eframe template to setup a egui application.

## Installation instructions (Optional)

- Clone the repository
- In a terminal navigate into the `script_setup` folder at the root of the repo
- run
  ```sh
  cargo install --path .
  ```
- Generate a config
  ```sh
  egui_template_copy --generate-config
  ```
- Edit the config as applicable ensuring to set the source directory to a fully qualified path so that it will work from anywhere

## Setup for first run

- Clone the repo to disk
- Generate a starter config file
  - If installed
    ```sh
    egui_template_copy --generate-config
    ```
  - If not installed
    - Navigate to the `script_setup` folder at the root of the repo
      ```sh
      cargo run -- --generate-config
      ```
- It will tell you where the config was created, please edit the config to match your use case
- Then run the program see built in application help for more info `-h` for quick version or `--help` for more detailed version.

## Create a new project from the template

- Ensure [Setup for first run](#setup-for-first-run) have been done
  - Note where the config is located as you need to edit it for the current project.
    If you don't know just run the generate command again it will tell you in the error message.
- Edit the config to match your current use case
