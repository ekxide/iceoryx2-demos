# iceoryx2-demos

This repository demonstrates the various topologies in which iceoryx2 is used.
It is a living project, and more topologies will be added over time.
However, it currently includes only the `automotive-reference-system`.

Please note that this repo is not intended as a tutorial for building systems based on iceoryx2.
Instead, it runs a single application configured with different command line arguments to create specific setups that can be introspected with [ekxide Mission Control](https://ekxide.io/products/mission-control/).

## Run the `automotive-reference-system`

This demo includes a `bash` script primarily for Linux.
It is compatible with most Unix-like systems.
On Windows, the script can be run using `WSL` or `Git Bash`.

Running the demo:
```sh
git clone https://github.com/ekxide/iceoryx2-demos.git
cd iceoryx2-demos
./automotive-reference-system.sh
```

Building the demo occurs automatically on first run and may take some time.
Once complete, the application is launched multiple times with various CLI arguments to demonstrate a typical automotive scenario.

<img width="1955" height="1293" alt="mico-01" src="https://github.com/user-attachments/assets/8589cd2e-9145-4b92-a94c-9eeeb50d5af3" />
