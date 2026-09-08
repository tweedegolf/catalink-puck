# CATALINK Precursor app
This is the CATALINK precursor app.

## Building
This is written to integrate with the precursor kernel like other apps in the xous kernel.
In order to build it it needs to be symlinked or copied into the apps folder of the xous-core repo.
### Setup
Follow these steps to integrate the app into the "xous-core" repo:
1. Clone the xous-core repo: [xous-core](https://github.com/betrusted-io/xous-core).
2. Copy or symlink the "catalink" folder into the "apps" folder of xous-core.
3. Add an entry for the app to the "manifest.json" file in the "apps" directory:
```json
"catalink": {
        "context_name": "CATALINK",
        "menu_name": {
            "appmenu.catalink": {
                "en": "CATALINK",
                "en-tts": "CATALINK"
            }
        }
    }
```
4. Add the crate to the "Cargo.toml" file in the root of the xous-core repo. It needs to be both in "default-members" and "members": `apps/catalink`.

## Running
### Running on the precursor
To build for the precursor run the following command in the root of the xous-core repo:
```
cargo xtask app-image-xip catalink
```
This will build an image of the kernel with the catalink app.
To open the app you will need to press the center key of the directional pad in the center, select "Switch app" and finally select "catalink".

### Running in the simulator
To run the app in the simulator simply run the following command:
```
cargo xtask run catalink
```
A window should open with the precursor interface.
To open the app you will need to press the "home" key on your keyboard, select "Switch app" and confirm with the "home" key, finally select "catalink" and confirm by pressing "home" again.

### Flashing to the precursor

In the `tools/` folder of xous-core, run `python3 usb_update.py -k`.

Before trying this the first time, you'll need to set up a few things:

- Set up a Python3 virtual environment:
      python3 -m venv venv

- Enter the Python3 virtual environment:
      source venv/bin/activate

- Install `cmake`, `yasm` using your distro package manager if you don't already have them.

- Install Python crypto components:

      pip install pycryptodome cryptography six

- Install other Python dependencies, we've identified at least these two on a more or less vanilla Ubuntu system:

      pip install pyusb progressbar2

- Add a `udev` rule to be able to access the precursor, by adding a file at `/etc/udev/rules.d/99-precursor-usb.rules` containing:

      SUBSYSTEM=="usb", ATTRS{idVendor}=="1209", ATTRS{idProduct}=="5bf0", GROUP="plugdev", TAG+="uaccess"

  Afterwards reload the `udev` rules using:

      sudo udevadm control --reload-rules
      sudo udevadm trigger

- Install the precursorupdater and update your device before flashing:

      pip install precursorupdater
      precursorupdater

For more information consult the README in the `tools/` folder.

### Running inside the Renode emulator

The normal simulator just runs the Xous app in a normal Rust environment that has been customized to *apear to be* a Precursor. But it
does not actually run the code that will be installed on the Precursor. To do that without actually flashing the device, use:

     cargo xtask renode-image catalink
     renode emulation/xous-release.resc

This will require renode, found at https://github.com/renode/. You may be able to install it from your package manager:

     paru -S renode (Arch Linux)
     brew install renode/tap/renode (MacOS)
