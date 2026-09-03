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
4. Add the crate to the "Cargo.toml" file in the root of the xous-core repo. It needs to be both in "default-members" and "members": `services/gam/src/api.rs`.


## Running
### Running on the precursor
To build for the precursor run the following command in the root of the xous-core repo:
```
cargo xtask app-image-xip catalink
```
This will build an image of the kernel with the catalink app.
To open the app you will need to press the center key of the directional pad in the center, select "Switch app" and finally select "catalink".

### Running in the emulator
To run the app in the emulator simply run the following command:
```
cargo xtask run catalink
```
A window should open with the precursor interface.
To open the app you will need to press the "home" key on your keyboard, select "Switch app" and confirm with the "home" key, finally select "catalink" and confirm by pressing "home" again.
