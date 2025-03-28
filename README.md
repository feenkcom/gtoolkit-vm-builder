# Glamorous Toolkit Virtual Machine Builder

This project holds a command line tool written in Rust to build and package [gtoolkit-vm](https://github.com/feenkcom/gtoolkit-vm), the virtual machine used by [Glamorous Toolkit](https://github.com/feenkcom/gtoolkit).

### Downloading sources and tools
First make sure to clone the `gtoolkit-vm`:
```
git clone https://github.com/feenkcom/gtoolkit-vm.git
cd gtoolkit-vm
```
Then download the latest released version of the `gtoolkit-vm-builder` for your platform inside the `gtoolkit-vm` folder:
```
curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/latest/download/gtoolkit-vm-builder-${TARGET}
chmod +x gtoolkit-vm-builder
```
`TARGET` is one of:
  - aarch64-apple-darwin
  - x86_64-apple-darwin
  - x86_64-unknown-linux-gnu
  - x86_64-pc-windows-msvc.exe

### Building and packaging a bundle

```
./gtoolkit-vm-builder \
    --release \
    --app-name 'GlamorousToolkit' \
    --identifier 'com.gtoolkit' \
    --author "feenk gmbh <contact@feenk.com>" \
    --libraries-versions libraries.version \
    --libraries cairo clipboard crypto freetype git gleam glutin pixels process sdl2 skia winit test-library
```

The resulting bundle will be created in the `target/${TARGET}/release/bundle` folder
