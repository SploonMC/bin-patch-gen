# bin-patch-gen 

**bin-patch-gen** is a tool designed to generate SpigotMC binary patches for all SpigotMC-supported Minecraft versions.

## Usage

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

#### Force Mode
```bash
./bin-patch-gen --force
```
Builds all versions regardless of whether they have already been built.

#### Clean Mode
```bash
./bin-patch-gen --clean
```
Removes all contents of the `run` directory. If the current working directory is named `run`, it will be cleared. This ensures a clean environment before starting the process.

**Note**: This may result in the deletion of important files, so use this option with caution.

#### Patch Mode
```bash
./bin-patch-gen patch oldfile newfile patchfile
```
This mode applies a patch to transform one file into another.  
- `oldfile`: The original file to be patched.  
- `patchfile`: The binary patch file.  
- `newfile`: The output file resulting from applying the patch.  

### Docker
There is a provided `docker-compose.yml` file which you can use. Our CI runs a cronjob with the provided `update.sh` script, which
just runs the docker container and pushes the patches to Git.

## Building
To build this project, you need Cargo and a recent version of Rust from the nightly channel. Building is straightforward, like any other Rust program:
```bash
cargo build
```

## Contributing
Any pull requests and issues are welcome.

All contributions should be:
- made to the dev branch.
- formatted using rustfmt.
- tested.

CI Contributions should only be done by maintainers.

## License
This project is licensed under the MIT license.
