# SwapBytes

SwapBytes is a simple peer to peer file sharing application that allows users to chat in public rooms, create private messaging sessions, and share files.

## How to run

To run the application, type the following into a terminal at the root of the directory:

```bash
cargo build
cargo run
```

To run multiple peers, run multiple terminal instances or run this on two seperate devices on the same network.  

## How to use

Once you must have more than one peer connected you can choose a username. From there you are brought to the Global chat topic.

### Changing Tabs

To change to other tabs (select room or dm), switch tabs by pressing tab.

### Select Room Tab

On the select room tab you can then select a room by using the arrow keys and pressing enter.

### Direct Messages Tab

On the direct messages tab you can then select a component using the ```~ (tilda)``` key.\n
When the input is highlighted yellow you can type and send messages to the selected user by pressing enter\n
When the People componenet is highlighted you can use the arrow keys and press enter to select who you wish to view and send direct messages too.\n
When the Incoming Requests component is highlighted you can use the arrow keys and press enter to provide the given file.


### Commands

The application has multiple commands that the user can use to perform different actions.

_Note that when using a command with an argument, you do not need to provide the square brackets. They are there to show that it is a variable._

Below is a list of the available commands:

**!create room [room]** - _Create a room with the name provided_   
**!request file [filename]** - _Request a file *Exactly* matching the filename provided from the currently selected peer_  