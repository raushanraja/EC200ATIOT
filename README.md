# EC200ATIOT
This project is a firmware for the ESP32-S3 microcontroller that implements. It implements AT basic structure for the Quectel EC200T IoT module. This also includes a relay controller, a PZEM004T sensor interface, and a subscription manager for MQTT messages.

## Features
- **AT Command Handling**: Implements a AT command parser and handler for the Quectel EC200T module.
- **Relay Controller**: Manages relay states based on received commands.
- **PZEM004T Sensor Interface**: Reads and processes data from the PZEM004T power sensor.
- **MQTT Subscription Manager**: Handles subscription messages and commands for MQTT communication.
- **Dev Containers Support**: Includes support for VS Code Dev Containers and GitHub Codespaces for a seamless development experience.

⚠️ Warning

This firmware is not optimized for production use. It is provided as-is, and using it in critical or high-risk applications is at your own risk. Please ensure thorough testing before deployment in any environment.

### Flash
- `cargo run`

### Project Key Files
- **`Cargo.toml`**: Contains the project dependencies and configuration.
- **`build.rs`**: Build script for the project.
- **`src/main.rs`**: Entry point of the application.
- **`src/at.rs`**: Contains the main AT module implementation.
- **`src/atcommands.rs`**: Defines AT commands and their implementations.
- **`src/atmodule.rs`**: Handles the state and events of the AT module.
- **`src/atres.rs`**: Processes responses from the AT module.
- **`src/constants.rs`**: Defines constants used throughout the project.
- **`src/controller.rs`**: Implements the relay controller.
- **`src/emon.rs`**: Handles communication with the PZEM004T sensor.
- **`src/subscribe.rs`**: Manages subscription messages and commands.
- **`scripts/build.sh`**: Script to build the project.
- **`scripts/flash.sh`**: Script to flash the firmware.
- **`wokwi.toml`**: Configuration for Wokwi simulation.



The emon rust code is modified & taken from [this repo](https://github.com/iostapyshyn/pzem004t)


## License
This project is licensed under the MIT License. See the LICENSE file for details.
MIT license [LICENSE-MIT](http://opensource.org/licenses/MIT)
