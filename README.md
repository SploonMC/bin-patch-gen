# bin-patch-gen 

**bin-patch-gen** is a tool designed to generate binary patches for all Minecraft versions supported by Spigot.

## Usage

### Manual Usage

#### Default Mode
```bash
./bin-patch-gen
```
This runs the program in its default mode. It retrieves a list of all versions that Spigot's BuildTools can build, iterates through them, and builds each version. After building, it generates a `bsdiff`/`bspatch` compatible patch file that can convert a vanilla server jar into a Spigot server jar.

#### Specific Version
```bash
./bin-patch-gen --version x.xx.x
```
This runs the program to generate a binary patch for a specific Minecraft version. Replace `x.xx.x` with the desired version.  
For example:  
```bash
./bin-patch-gen --version 1.21.3
```
Generates a patch for version 1.21.3.

#### Patch Mode
```bash
./bin-patch-gen patch oldfile newfile patchfile
```
This mode applies a patch to transform one file into another.  
- `oldfile`: The original file to be patched.  
- `patchfile`: The binary patch file.  
- `newfile`: The output file resulting from applying the patch.  

### Docker
Insert documentation here.

## Building
To build this project, you need Cargo and a recent version of Rust from the nightly channel. Building is straightforward, like any other Rust program:
```bash
cargo build
```

## Contributing
Insert documentation here.

## License
Insert documentation here.
