# iceoryx2-demos

This repository demonstrates the various topologies in which iceoryx2 is used.
It is a living project, and more topologies will be added over time.
However, it currently includes only the `automotive-reference-system`.

Please note that this repo is not intended as a tutorial for building systems based on iceoryx2.
Instead, it runs a single application configured with different command line arguments to create specific setups that can be introspected with [ekxide Mission Control](https://ekxide.io/products/mission-control/).
The repository is therefore also serves as a showcase for `ekxide Mission Control` features.

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

<img width="1955" height="1293" alt="node overview" src="https://github.com/user-attachments/assets/8589cd2e-9145-4b92-a94c-9eeeb50d5af3" />

## Introspect the `automotive-reference-system`

With `ekxide Mission Control`, you get a clear overview of all running nodes and services.

As shown in the image above, below the node graph there is a node list displaying the state of each node, CPU load, memory usage, thread count, and more.

For demonstration purposes, the application runs a background thread to simulate load and the `Behavior Planner` simulates a process that consumes an entire CPU core on its own.
This is visible both in the node list and as a bar inside the node in the graph.

Clicking on a node in the graph expands it and reveals additional information.
When expanded, you can send a signal to the corresponding process (e.g., `SIGTERM` or `SIGKILL`) via a context menu that opens when clicking the node name.
The screenshot below shows the `Behavior Planner` after a `SIGKILL` was sent to the process.

<img width="1955" height="1293" alt="node overview with expanded dead node" src="https://github.com/user-attachments/assets/6012ea92-b9fd-4896-82a4-fd42ca4793f1" />

As you can see, a dead node is marked in red, and all connections are also highlighted in red.
The node list labels the node status as `Dead`.

The next image shows the service list.
All discovered services are presented with detailed information, such as the messaging pattern and number of senders and receivers.
Clicking on an entry expands it and reveals more details.
Depending on the messaging pattern, a histogram also displays the frequency of published messages.

<img width="1955" height="1293" alt="service overview with analytics of a service" src="https://github.com/user-attachments/assets/8f4aedde-554e-4ebb-97f9-9aa5ea944d10" />

By default, the expanded view shows the `Analytics` tab, but the dropdown menu also offers the option to display the `Configuration` of the service.

<img width="1955" height="164" alt="service overview with the configuration of a service" src="https://github.com/user-attachments/assets/98e60a23-4742-4d76-a4cd-c183240f856e" />

The last tab is the history, which provides an overview of all detected events during the runtime of `ekxide Mission Control`.

<img width="1955" height="1293" alt="history" src="https://github.com/user-attachments/assets/d2215cad-6a75-46c3-86db-d13ca3ad729e" />

The views are fully configurable. For example, a second node graph can be added to get a better overview on a large system.

<img width="1955" height="1293" alt="node overview with custom panel" src="https://github.com/user-attachments/assets/8d74364e-c185-4fb3-bbd0-8d9fd9e32337" />

The setup described above can be created by right-clicking into the window and selecting `iceoryx2 introspection -> add -> node graph`.
The added graph can then be moved (`move pane` in the right-click menu on the pane) to the desired location.
By default, the node graph shows the `Named` level of detail.
For the image above, the `level of detail` was changed to `Minimalistic` in the right-click menu, and the zoom slider in the bottom-left corner was used to fit the graph into the pane.

The current version of `ekxide Mission Control` does not yet support saving custom view setups.
This feature will be included in a future update.

As you may have noticed, the right-click menu also includes the entry `iceoryx2 introspection -> cleanup dead nodes`.
This can be used to manually clean up resources for detected dead nodes.
By default, cleanup happens automatically when a new node is created or gracefully shuts down, assuming the process has the necessary permissions.
